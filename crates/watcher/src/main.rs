use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    sync::mpsc::{self, Sender},
    thread::{self, Thread},
    time::{Duration, Instant},
};
use tokio::{
    runtime::Handle,
    spawn,
    task::{spawn_local, LocalSet},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

use event_store::{core::event_bus::EventBus, prelude::PostgresEventBus};
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
struct App {
    state: TableState,
    items: Vec<Vec<String>>,
}

impl App {
    fn new() -> App {
        App {
            state: TableState::default(),
            items: vec![],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

enum Message {
    Interrupt,
    Tick,
    Event(Vec<String>),
}

async fn run_stream(tx: Sender<Message>) {
    let mut listener = PostgresEventBus::initiate(
        "postgresql://postgres:postgres@localhost/event_store_bank".to_string(),
    )
    .await
    .unwrap();

    let mut stream = listener.create_stream().await;
    while let Ok(Some(notif)) = stream.try_next().await {
        tx.send(Message::Event(vec![format!("{:?}", notif)]))
            .expect("Cannot send notif");
    }
}

async fn run_app<'a, B: Backend>(terminal: &'a mut Terminal<B>, mut app: App) -> io::Result<()> {
    // let (tx, mut rx) = channel::<Message>(1024);
    //
    // let tx_two = tx.clone();
    // thread::spawn(move || loop {
    //     let local = LocalSet::new();
    //     local.run_until(async {
    //         loop {
    //             if let Ok(Event::Key(key)) = event::read() {
    //                 match key.code {
    //                     KeyCode::Char('q') => {
    //                         tx_two.send(Message::Interrupt).await;
    //                     }
    //                     // KeyCode::Down => app.next(),
    //                     // KeyCode::Up => app.previous(),
    //                     _ => {}
    //                 }
    //             }
    //         }
    //     });
    // });
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
    tokio::pin!(fut);
    let handle = Handle::current();
    thread::spawn(move || {
        handle.spawn(fut);
    });
    // tokio::spawn(async move {
    //     let local = LocalSet::new();
    //     local
    //         .run_until(async {
    //             spawn_local(fut).await;
    //         })
    //         .await;
    // });

    terminal.draw(|f| ui(f, &mut app))?;

    loop {
        match rx.try_recv() {
            Ok(Message::Interrupt) => {
                println!("received Interrupt");
                return Ok(());
            }
            Ok(Message::Event(values)) => {
                println!("received events");
                app.items.push(values);
            }
            _ => {}
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(5)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Header1", "Header2", "Header3"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.as_str()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}
