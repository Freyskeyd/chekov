use tui::style::{Color, Style};

use crate::app::App;

pub fn get_color() -> Style {
    Style::default().fg(Color::Gray)
}

pub fn get_main_layout_margin(app: &App) -> u16 {
    // if app.size.height > SMALL_TERMINAL_HEIGHT {
    1
    // } else {
    //   0
    // }
}
