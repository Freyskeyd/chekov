use crossterm::event::{self, Event, KeyCode};
use std::{
    io,
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use tui::{backend::Backend, Terminal};
use uuid::Uuid;

use event_store::{
    core::{backend::Backend as EventStoreBackend, event_bus::EventBus},
    prelude::{PostgresBackend, PostgresEventBus, RecordedEvent},
};
use futures::TryStreamExt;

use crate::{ui::ui, Message};

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

pub struct App<'a> {
    pub(crate) title: &'a str,
    pub(crate) tabs: TabsState<'a>,
    pub(crate) items: Vec<String>,
}

impl<'a> App<'a> {
    pub(crate) fn new() -> App<'a> {
        App {
            title: "Chekov",
            tabs: TabsState {
                titles: vec!["Events", "Streams"],
                index: 0,
            },
            items: vec![],
        }
    }
}

async fn run_stream(tx: Sender<Message>) -> Result<(), ()> {
    let backend =
        PostgresBackend::with_url("postgresql://postgres:postgres@localhost/event_store_bank")
            .await
            .unwrap();

    let mut listener = PostgresEventBus::initiate(
        "postgresql://postgres:postgres@localhost/event_store_bank".to_string(),
    )
    .await
    .unwrap();

    let mut stream = listener.create_stream().await;
    while let Ok(Some(notif)) = stream.try_next().await {
        // Reload state

        match notif {
            event_store::core::event_bus::EventBusMessage::Notification(notification)
                if notification.stream_uuid == "$all" =>
            {
                let stream_uuid = notification.stream_uuid;
                let correlation_id = Uuid::new_v4();

                if let Ok(events) = backend
                    .read_stream(
                        stream_uuid.to_string(),
                        notification.first_stream_version as usize,
                        (notification.last_stream_version - notification.first_stream_version + 1)
                            as usize,
                        correlation_id,
                    )
                    .await
                {
                    let e = events
                        .into_iter()
                        .map(|event: RecordedEvent| {
                            format!(
                                "{event_type}({})",
                                event.data,
                                event_type = event.event_type
                            )
                        })
                        .collect();

                    tx.send(Message::Event(e)).expect("Cannot send notif");
                }
            }
            event_store::core::event_bus::EventBusMessage::Notification(_) => {}
            event_store::core::event_bus::EventBusMessage::Events(_stream, _events) => todo!(),
            event_store::core::event_bus::EventBusMessage::Unkown => {}
        }
    }
    Ok(())
}

pub(crate) async fn run_app<'a, B: Backend>(
    terminal: &'a mut Terminal<B>,
    mut app: App<'_>,
) -> io::Result<()> {
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    let tx_one = tx.clone();
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let Event::Key(key) = event::read().expect("can read events") {
                    if let KeyCode::Char('q') = key.code {
                        tx_one.send(Message::Interrupt).expect("can send events");
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate && tx_one.send(Message::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    let fut = run_stream(tx);
    let handle = Handle::current();

    thread::spawn(move || {
        handle.spawn(fut);
    });

    terminal.draw(|f| ui(f, &mut app))?;

    loop {
        match rx.try_recv() {
            Ok(Message::Interrupt) => {
                return Ok(());
            }
            Ok(Message::Event(mut values)) => {
                app.items.append(&mut values);
                terminal.draw(|f| ui(f, &mut app))?;
            }
            _ => {}
        }
    }
}
