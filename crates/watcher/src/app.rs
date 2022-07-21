use crossterm::event::{self, Event, KeyCode};
use std::{
    io,
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use tonic::transport::Uri;
use tui::{
    backend::Backend,
    widgets::{ListState, TableState},
    Terminal,
};
use uuid::Uuid;

use event_store::prelude::RecordedEvent;

use crate::{conn, state::State, ui::ui, Message};

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub struct App<'a> {
    pub(crate) title: &'a str,
    pub(crate) tabs: TabsState<'a>,
    pub(crate) state: TableState,
    pub(crate) list_state: ListState,
    pub(crate) events: Vec<String>,
    pub(crate) streams: Vec<String>,
}

impl<'a> App<'a> {
    pub(crate) fn new() -> App<'a> {
        App {
            state: TableState::default(),
            title: "Chekov",
            list_state: ListState::default(),
            tabs: TabsState {
                titles: vec!["Events", "Streams"],
                index: 0,
            },
            events: vec![],
            streams: vec![],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.events.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.events.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

async fn run_stream(tx: Sender<Message>) -> Result<(), ()> {
    let target = "http://[::1]:50051";
    let uri = Uri::from_static(target);
    let mut conn = conn::Connection::new(uri);
    let mut state = State::default();

    loop {
        tokio::select! {
            state_update = conn.next_update() => {
                state.update(state_update);
            }
        }
    }

    // let mut stream = listener.create_stream().await;
    // while let Ok(Some(notif)) = stream.try_next().await {
    //     // Reload state
    //
    //     match notif {
    //         event_store::core::event_bus::EventBusMessage::Notification(notification)
    //             if notification.stream_uuid == "$all" =>
    //         {
    //             let stream_uuid = notification.stream_uuid;
    //             let correlation_id = Uuid::new_v4();
    //
    //             if let Ok(events) = backend
    //                 .read_stream(
    //                     stream_uuid.to_string(),
    //                     notification.first_stream_version as usize,
    //                     (notification.last_stream_version - notification.first_stream_version + 1)
    //                         as usize,
    //                     correlation_id,
    //                 )
    //                 .await
    //             {
    //                 let e = events
    //                     .into_iter()
    //                     .map(|event: RecordedEvent| {
    //                         format!(
    //                             "{event_type}({})",
    //                             event.data,
    //                             event_type = event.event_type
    //                         )
    //                     })
    //                     .collect();
    //
    //                 tx.send(Message::Event(e)).expect("Cannot send notif");
    //             }
    //         }
    //         event_store::core::event_bus::EventBusMessage::Notification(_) => {}
    //         event_store::core::event_bus::EventBusMessage::Events(stream, events) => todo!(),
    //         event_store::core::event_bus::EventBusMessage::Unkown => {}
    //     }
    // }
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
                    match key.code {
                        KeyCode::Char('q') => {
                            tx_one.send(Message::Interrupt).expect("can send events");
                        }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx_one.send(Message::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let tx_two = tx.clone();
    let fut = run_stream(tx_two.clone());
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
                app.events.append(&mut values);
                terminal.draw(|f| ui(f, &mut app))?;
            }
            _ => {}
        }
    }
}
