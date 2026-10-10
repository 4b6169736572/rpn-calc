use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    DefaultTerminal,
};

use crate::{domain, stack::Stack, ui};

#[derive(Default, Debug)]
pub struct App {
    stack: Stack,
    input: String,
    should_quit: bool,
}

impl App {
    pub fn push_input(&mut self) {
        self.stack.push(self.input.into::<f64>()?);
    }
}

// Every type of message that handle_key() can return.
#[derive(Debug, PartialEq)]
pub enum Message {
    Type(char),
    Return,
    Backspace,
    Quit,
}

// Main begins the loop.
pub fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::default();
    while !app.should_quit {
        // terminal frame and ref to app are passed to view() which handles rendering the app state.
        terminal.draw(|frame| ui::view(frame, &app))?;
        // TODO: Remove event handling from here.
        //
        // If there is an event which is a key press, pass it to handle_key().
        // If handle_key() returns a message, pass the message and a mut ref of app to update(),
        // otherwise, skip to next loop.
        if let Event::Key(key) = event::read()?
            && let Some(msg) = handle_key(key)
        {
            tracing::debug!(?msg, "update");
            update(&mut app, msg);
        }
    }
    Ok(())
}

// TODO: make into general event handler, and extract into seperate file.
pub fn handle_key(key: KeyEvent) -> Option<Message> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Message::Quit);
    }
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char(c) => Some(Message::Type(c)),
        KeyCode::Backspace => Some(Message::Backspace),
        KeyCode::Enter => Some(Message::Return),
        _ => None,
    }
}

pub fn update(app: &mut App, msg: Message) {
    match msg {
        Message::Type(c) => app.input.push(c),
        Message::Backspace => {
            app.stack.pop();
        }
        Message::Return => app.push_input(),
        Message::Quit => app.should_quit = true,
    }
}
