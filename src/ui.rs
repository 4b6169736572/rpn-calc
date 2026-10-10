use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::app::App;

pub fn view(frame: &mut Frame, app: &App) {
    let [body, input, footer] = Layout::vertical([
        Constraint::fill(1),
        Constraint::length(3),
        constraint::length(3),
    ])
    .areas(frame.area());

    let [index, stack_area] = Layout::horizontal([
        Constraint::length(3),
        Constraint::fill(1),
        //TODO:  Constraint::length(5), history
    ])
    .areas(body);

    //TODO: turn Stack into vec of Lines.
    let stack_lines = app.stack.clone().iter().map(|item| Line::from(item));
    let stack_widget = Paragraph::new(stack_lines).block(Block::bordered().title(" Stack "));

    //TODO: history.
    // let history_lines = todo!();
    // let history_area = Paragraph::new(history_lines).block(Block::bordered().title(" History "));

    frame.render_widget(stack_widget, stack_area);
}
