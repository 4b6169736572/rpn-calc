use std::{collections::HashMap, ops::Div};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};

use crate::{
    app::{GroupingSymbol::*, Mode::Normal, StackObject::*, Symbol::Variable},
    domain,
    errors::Error,
    ui,
};

#[derive(Default)]
pub struct App {
    pub stack: Vec<StackObject>,
    pub input: Input,
    pub should_quit: bool,
    pub variables: HashMap<char, StackObject>,
    pub mode: Mode,
}

impl App {
    pub fn push_input(&mut self) {
        match self.input.parse() {
            Ok(object) => self.stack.push(object),
            Err(errror) => panic!(),
        }
    }
}

#[derive(Default, Clone)]
pub enum Mode {
    #[default]
    Normal,
    Store,
    Recall,
    Alpha,
    Repeat(u16),
    IntoRepeat,
}

#[derive(Clone, Default)]
pub struct Input(String);
impl Input {
    fn push(&mut self, character: char) {
        self.0.push(character);
    }
    fn pop(&mut self) {
        self.0.pop();
    }
    fn parse(&mut self) -> Result<StackObject, Error> {
        if let Ok(num) = self.0.parse::<f64>() {
            self.0.clear();
            Ok(Number(num))
        } else {
            Err(Error::StackError("Parsing Error"))
        }
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone)]
pub enum StackObject {
    Number(f64),
    Expression(Expression),
    Symbol(Symbol),
}

impl StackObject {
    pub fn render(&self, depth: usize) -> String {
        match self {
            Number(num) => num.to_string(),
            Expression(expression) => {
                let left = expression.left.render(depth + 1);
                let right = expression.right.render(depth + 1);
                let rendered = expression.function.render(left, right);
                if depth == 0 {
                    rendered
                } else {
                    format!("({rendered})")
                }
                // else if depth % 2 == 1 {
                //     format!("({})", rendered)
                // } else {
                //     format!("[{}]", rendered)
                // }
            }
            Symbol(symbol) => symbol.render(),
        }
    }
}

impl StackObject {
    fn evaluate(self, app: &App) -> Self {
        match self {
            Expression(expression) => {
                if let Ok(result) = expression.evaluate(app) {
                    result
                } else {
                    unreachable!()
                }
            }
            Symbol(symbol) => {
                if let Variable(c) = symbol {
                    if let Some(object) = app.variables.get(&c) {
                        object.clone()
                    } else {
                        Symbol(Variable(c))
                    }
                } else {
                    Symbol(symbol)
                }
            }
            Number(num) => self,
        }
    }
}

impl From<StackObject> for String {
    fn from(object: StackObject) -> Self {
        object.render(0)
    }
}

#[derive(Clone)]
pub struct Expression {
    left: Box<StackObject>,
    right: Box<StackObject>,
    function: Box<BinaryFunction>,
}

use BinaryFunction::*;
use StackObject::*;

impl Expression {
    pub fn evaluate(self, app: &App) -> Result<StackObject, Error> {
        match (*self.left, *self.right, *self.function) {
            // Number | Number | Function |
            (Number(left), Number(right), function) => match function {
                Add => Ok(Number(left + right)),
                Sub => Ok(Number(left - right)),
                Div => {
                    if right == 0f64 {
                        Err(Error::DivByZero)
                    } else {
                        Ok(Number(left / right))
                    }
                }
                Mult => Ok(Number(left * right)),
                Exp => Ok(Number(left.powf(right))),
            },
            (left, right, function) => Ok(Expression(Expression {
                left: Box::new(left.evaluate(app)),
                right: Box::new(right.evaluate(app)),
                function: Box::new(function),
            })
            .evaluate(app)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub enum Symbol {
    Variable(char),
    Left(GroupingSymbol),
    Right(GroupingSymbol),
}

impl Symbol {
    pub fn evaluate(self, app: &App) -> StackObject {
        match self {
            Symbol::Variable(c) => {
                if let Some(object) = app.variables.get(&c) {
                    object.clone()
                } else {
                    Symbol(self)
                }
            }
            _ => Symbol(self),
        }
    }
    pub fn render(&self) -> String {
        match self {
            Symbol::Variable(char) => char.to_string(),
            Symbol::Left(grouper) => grouper.left(),
            Symbol::Right(grouper) => grouper.right(),
        }
    }
}

#[derive(Clone)]
pub enum GroupingSymbol {
    Paren,
    Bracket,
    Angle,
    Curly,
}

impl GroupingSymbol {
    pub fn left(&self) -> String {
        match self {
            GroupingSymbol::Paren => String::from("("),
            GroupingSymbol::Bracket => String::from("["),
            GroupingSymbol::Angle => String::from("<"),
            GroupingSymbol::Curly => String::from("{"),
        }
    }
    pub fn right(&self) -> String {
        match self {
            GroupingSymbol::Paren => String::from(")"),
            GroupingSymbol::Bracket => String::from("]"),
            GroupingSymbol::Angle => String::from(">"),
            GroupingSymbol::Curly => String::from("}"),
        }
    }
}

#[derive(Clone)]
pub enum BinaryFunction {
    Add,
    Sub,
    Div,
    Mult,
    Exp,
}

impl BinaryFunction {
    fn render(&self, left: String, right: String) -> String {
        match self {
            Add => format!("{left} + {right}"),
            Sub => format!("{left} - {right}"),
            Div => format!("{left} / {right}"),
            Mult => format!("{left} * {right}"),
            Exp => format!("{left}^{right}"),
        }
    }
}

// Every type of message that handle_key() can return.
#[derive(Clone)]
pub enum Message {
    Flip,
    Numerical(char),
    Function(BinaryFunction),
    Return,
    Backspace,
    Evaluate,
    Quit,
    ChangeMode(Mode),
    Store(char),
    Recall(char),
    Alpha(char),
    Repeat(u16),
    IntoRepeat,
}

// Main begins the loop.
pub fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::default();
    while !app.should_quit {
        // terminal frame and ref to app are passed to view() which handles rendering the app state.
        terminal.draw(|frame| ui::view(frame, &app))?;
        // If there is an event which is a key press, pass it to handle_key().
        // If handle_key() returns a message, pass the message and a mut ref of app to update(),
        // otherwise, skip to next loop.
        if let Event::Key(key) = event::read()?
            && let Some(msg) = handle_key(key, &app.mode)
        {
            update(&mut app, msg);
        }
    }
    Ok(())
}

const NUMERICAL: &str = "0123456789.";
const CHARACTER: &str = "abcdfghijklmnopqrstuvwxyzABCDFGHIJKLMNOPQRSTUVWXYZ";

pub fn handle_key(key: KeyEvent, mode: &Mode) -> Option<Message> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    // if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
    //     return Some(Message::Quit);
    // }
    if let KeyCode::Char(c) = key.code
        && key.modifiers.contains(KeyModifiers::CONTROL)
    {
        match c {
            'n' => {
                return Some(Message::IntoRepeat);
            }
            // num if NUMERICAL.contains(c) => {
            //     return Some(Message::Repeat(num.to_digit(10).unwrap() as u16));
            // }
            'c' => {
                return Some(Message::Quit);
            }
            _ => {
                return None;
            }
        }
    }
    // if key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Char('=') {
    //     return Some(Message::Result);
    // }
    match mode {
        Mode::Normal | Mode::Repeat(_) => match key.code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char(c) => match c {
                c if NUMERICAL.contains(c) => Some(Message::Numerical(c)),
                '/' => Some(Message::Function(Div)),
                '*' => Some(Message::Function(Mult)),
                '+' => Some(Message::Function(Add)),
                '-' => Some(Message::Function(Sub)),
                '^' => Some(Message::Function(Exp)),
                '=' => Some(Message::Evaluate),
                'r' => Some(Message::ChangeMode(Mode::Recall)),
                's' => Some(Message::ChangeMode(Mode::Store)),
                '\'' => Some(Message::ChangeMode(Mode::Alpha)),
                _ => None,
            },
            KeyCode::Backspace => Some(Message::Backspace),
            KeyCode::Enter => Some(Message::Return),
            KeyCode::Esc => Some(Message::ChangeMode(Normal)),
            KeyCode::Tab => Some(Message::Flip),
            _ => None,
        },
        Mode::Store => match key.code {
            KeyCode::Char(c) if CHARACTER.contains(c) => Some(Message::Store(c)),
            KeyCode::Backspace => Some(Message::Backspace),
            _ => None,
        },
        Mode::Recall => match key.code {
            KeyCode::Char(c) if CHARACTER.contains(c) => Some(Message::Recall(c)),
            KeyCode::Backspace => Some(Message::Backspace),
            _ => None,
        },
        Mode::Alpha => match key.code {
            KeyCode::Char(c) if CHARACTER.contains(c) => Some(Message::Alpha(c)),
            KeyCode::Backspace => Some(Message::Backspace),
            _ => None,
        },
        Mode::IntoRepeat => match key.code {
            KeyCode::Char(num) if NUMERICAL.contains(num) => {
                Some(Message::Repeat(num.to_digit(10).unwrap() as u16))
            }
            _ => None,
        },
    }
}

pub fn update(app: &mut App, msg: Message) {
    match msg {
        Message::Numerical(c) => app.input.push(c),
        Message::Function(function) => {
            if !app.input.is_empty() {
                app.push_input();
            }
            let (right, left) = (app.stack.pop(), app.stack.pop());
            match (left, right) {
                (Some(left), Some(right)) => app.stack.push(Expression(Expression {
                    left: Box::new(left),
                    right: Box::new(right),
                    function: Box::new(function),
                })),
                (None, Some(object)) => app.stack.push(object),
                _ => {}
            }
        }
        Message::Backspace => {
            if let Mode::Repeat(num) = app.mode {
                for _ in 1..num {
                    app.stack.push(app.stack.last().cloned().unwrap())
                }
                app.mode = Normal;
            } else {
                app.stack.pop();
            }
        }
        Message::Return => {
            if !app.input.is_empty() {
                app.push_input();
            } else {
                if let Some(object) = app.stack.last().cloned() {
                    app.stack.push(object);
                }
            }
            if let Mode::Repeat(num) = app.mode {
                for _ in 1..num {
                    app.stack.push(app.stack.last().cloned().unwrap())
                }
                app.mode = Normal;
            }
        }
        Message::Quit => app.should_quit = true,
        Message::Store(c) => {
            if !app.input.is_empty() {
                app.push_input();
                app.variables.insert(c, app.stack.pop().unwrap());
            } else if let Some(object) = app.stack.last() {
                app.variables.insert(c, object.clone());
            }
            app.mode = Normal
        }
        Message::Flip => {
            if let Mode::Repeat(num) = app.mode
                && num < app.stack.len() as u16
            {
                let mut buf: Vec<StackObject> = Vec::new();
                for i in 1..num {
                    buf.push(app.stack.pop().unwrap());
                }
                let last = app.stack.pop().unwrap();
                buf.into_iter()
                    .rev()
                    .map(|object| app.stack.push(object))
                    .collect::<()>();
                app.stack.push(last);
            } else {
                match (app.stack.pop(), app.stack.pop()) {
                    (Some(bottom), Some(top)) => {
                        app.stack.push(bottom);
                        app.stack.push(top);
                    }
                    (Some(bottom), None) => {
                        app.stack.push(bottom);
                    }
                    _ => {}
                }
            }
            app.mode = Normal;
        }
        Message::ChangeMode(mode) => app.mode = mode,
        Message::Evaluate => {
            if let Some(object) = app.stack.pop() {
                match object {
                    Expression(expression) => {
                        app.stack.push(expression.evaluate(&*app).unwrap());
                    }
                    Symbol(Symbol::Variable(c)) => {
                        if let Some(result) = app.variables.get(&c) {
                            app.stack.push(result.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        Message::Recall(c) => {
            if let Some(object) = app.variables.get(&c) {
                app.stack.push(object.clone());
            }
            app.mode = Normal;
        }
        Message::Alpha(c) => {
            app.stack.push(Symbol(Variable(c)));
            app.mode = Normal;
        }
        Message::IntoRepeat => {
            app.mode = Mode::IntoRepeat;
        }
        Message::Repeat(num) => {
            app.mode = Mode::Repeat(num);
        }
    }
}
