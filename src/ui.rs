use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Padding, Paragraph},
};

use crate::app::App;

const INPUT_SET: ratatui::symbols::border::Set = ratatui::symbols::border::Set {
    top_left: "┌",
    top_right: "┐",
    bottom_left: "└",
    bottom_right: "┘",
    vertical_left: ">",
    vertical_right: "│",
    horizontal_top: "─",
    horizontal_bottom: "─",
};

pub fn view(frame: &mut Frame, app: &App) {
    let [body_area, input_area, footer_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let [index_area, stack_area, history_area] = Layout::horizontal([
        Constraint::Length(3),
        Constraint::Fill(2),
        Constraint::Min(15),
    ])
    .areas(body_area);

    let stack_lines: Vec<Line> = app
        .stack
        .clone()
        .iter()
        .map(|item| Line::from(item.render(0)))
        .collect();
    let stack_border_widget = Block::bordered()
        .title(" Stack ")
        .padding(Padding::new(2, 2, 0, 0));

    let stack_inner_area = stack_border_widget.inner(stack_area);
    let stack_widget = Paragraph::new(stack_lines);
    let used_stack_height = app.stack.len() as u16;
    let [stack_bottom] = Layout::vertical([Constraint::Length(used_stack_height)])
        .flex(Flex::End)
        .areas(stack_inner_area);
    let [index_bottom, _] =
        Layout::vertical([Constraint::Length(used_stack_height), Constraint::Length(1)])
            .flex(Flex::End)
            .areas(index_area);
    let [_, usable_index] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
        .flex(Flex::End)
        .areas(index_area);

    let input_widget = Paragraph::new(app.input.string()).block(
        Block::bordered()
            .border_set(INPUT_SET)
            .title(" Input ")
            .padding(Padding::new(2, 2, 0, 0)),
    );

    let history_widget = Block::bordered().title(" History ");

    let help_widget = Block::bordered().title(" Help ");

    let index_widget_nums = Paragraph::new(
        (1..=stack_inner_area.height)
            .map(|num| Line::from(num.to_string()))
            .rev()
            .collect::<Vec<Line>>(),
    )
    .right_aligned()
    .block(Block::default().padding(Padding::new(0, 1, 0, 0)));

    //TODO: history.
    // let history_lines = todo!();
    // let history_area = Paragraph::new(history_lines).block(Block::bordered().title(" History "));

    frame.render_widget(stack_border_widget, stack_area);
    frame.render_widget(stack_widget, stack_bottom);
    frame.render_widget(input_widget, input_area);
    frame.render_widget(history_widget, history_area);
    frame.render_widget(help_widget, footer_area);
    frame.render_widget(index_widget_nums, usable_index);
}
