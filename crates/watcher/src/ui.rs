use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Tabs},
    Frame,
};

use crate::app::App;
mod util;
use util::get_color;

fn draw_selectable_list<B, S>(
    f: &mut Frame<B>,
    _app: &App,
    layout_chunk: Rect,
    title: &str,
    items: &[S],
    _highlight_state: (bool, bool),
    selected_index: Option<usize>,
) where
    B: Backend,
    S: std::convert::AsRef<str>,
{
    let mut state = ListState::default();
    state.select(selected_index);

    let lst_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(Span::raw(i.as_ref())))
        .collect();

    let list = List::new(lst_items)
        .block(
            Block::default()
                .title(Span::styled(title, get_color()))
                .borders(Borders::ALL)
                .border_style(get_color()),
        )
        .style(Style::default().fg(Color::Reset))
        .highlight_style(get_color().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, layout_chunk, &mut state);
}

fn draw_main_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let constraints = vec![Constraint::Percentage(50), Constraint::Percentage(50)];
    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Vertical)
        .split(area);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .direction(Direction::Horizontal)
            .split(chunks[0]);

        let events = app.events.clone();
        let streams = app.streams.clone();

        draw_selectable_list(f, app, chunks[0], "Events", &events, (false, false), None);
        draw_selectable_list(f, app, chunks[1], "Streams", &streams, (false, false), None);
    }
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .direction(Direction::Horizontal)
            .split(chunks[1]);
        let logs = app.events.clone();

        draw_selectable_list(f, app, chunks[0], "Aggregates", &logs, (false, false), None);
        draw_selectable_list(
            f,
            app,
            chunks[1],
            "EventHandlers",
            &logs,
            (false, false),
            None,
        );
    }
}

pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(app.title))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);

    f.render_widget(tabs, rects[0]);

    match app.tabs.index {
        0 => draw_main_tab(f, app, rects[1]),
        // 1 => draw_second_tab(f, app, chunks[1]),
        // 2 => draw_third_tab(f, app, chunks[1]),
        _ => {}
    };
}
