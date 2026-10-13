use std::ops::Div;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};

use crate::{app::GroupingSymbol::*, app::StackObject::*, domain, errors::Error, ui};

#[derive(Default)]
pub struct App {
    pub stack: Vec<StackObject>,
    pub input: Input,
    pub should_quit: bool,
}

impl App {
    pub fn push_input(&mut self) {
        match self.input.parse() {
            Ok(object) => self.stack.push(object),
            Err(errror) => panic!(),
        }
    }
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
                } else if depth % 2 == 1 {
                    format!("( {} )", rendered)
                } else {
                    format!("[ {} ]", rendered)
                }
            }
            Symbol(symbol) => symbol.render(),
        }
    }
}

impl Into<String> for StackObject {
    fn into(self) -> String {
        self.render(0)
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
    pub fn evaluate(self) -> Result<StackObject, Error> {
        match (*self.left, *self.right, *self.function) {
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
            (Expression(left_expression), Expression(right_expression), function) => {
                match (left_expression.evaluate()?, right_expression.evaluate()?) {
                    (Expression(left_expression), Expression(right_expression)) => {
                        Ok(Expression(Expression {
                            left: Box::new(Expression(left_expression)),
                            right: Box::new(Expression(right_expression)),
                            function: Box::new(function),
                        }))
                    }
                    (left, right) => Expression {
                        left: Box::new(left),
                        right: Box::new(right),
                        function: Box::new(function),
                    }
                    .evaluate(),
                }
            }
            (Expression(left_expression), Number(right), function) => {
                match left_expression.evaluate()? {
                    Expression(left_expression) => Ok(Expression(Expression {
                        left: Box::new(Expression(left_expression)),
                        right: Box::new(Number(right)),
                        function: Box::new(function),
                    })),
                    Number(left) => Expression {
                        left: Box::new(Number(left)),
                        right: Box::new(Number(right)),
                        function: Box::new(function),
                    }
                    .evaluate(),
                    _ => Err(Error::StackError("Evaluating Symbol.")),
                }
            }
            (Number(left), Expression(right_expression), function) => {
                match right_expression.evaluate()? {
                    Expression(right_expression) => Ok(Expression(Expression {
                        left: Box::new(Number(left)),
                        right: Box::new(Expression(right_expression)),
                        function: Box::new(function),
                    })),
                    Number(right) => Expression {
                        left: Box::new(Number(left)),
                        right: Box::new(Number(right)),
                        function: Box::new(function),
                    }
                    .evaluate(),
                    _ => Err(Error::StackError("Evaluating Symbol.")),
                }
            }
            _ => Err(Error::StackError("Evaluating Symbol.")),
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
    pub fn render(&self) -> String {
        match self {
            Symbol::Variable(char) => char.to_string(),
            Symbol::Left(grouper) => String::from(grouper.left()),
            Symbol::Right(grouper) => String::from(grouper.right()),
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
pub enum Message {
    Numerical(char),
    Function(BinaryFunction),
    Return,
    Backspace,
    Evaluate,
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
            update(&mut app, msg);
        }
    }
    Ok(())
}

const NUMERICAL: &str = "0123456789.";

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
        KeyCode::Char(c) => match c {
            c if NUMERICAL.contains(c) => Some(Message::Numerical(c)),
            '/' => Some(Message::Function(Div)),
            '*' => Some(Message::Function(Mult)),
            '+' => Some(Message::Function(Add)),
            '-' => Some(Message::Function(Sub)),
            '^' => Some(Message::Function(Exp)),
            '=' => Some(Message::Evaluate),
            _ => None,
        },
        KeyCode::Backspace => Some(Message::Backspace),
        KeyCode::Enter => Some(Message::Return),
        _ => None,
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

            if let (Some(right), Some(left)) = (right, left) {
                app.stack.push(Expression(Expression {
                    left: Box::new(left),
                    right: Box::new(right),
                    function: Box::new(function),
                }))
            }
        }
        Message::Backspace => {
            app.stack.pop();
        }
        Message::Return => {
            if !app.input.is_empty() {
                app.push_input();
            } else {
                if let Some(object) = app.stack.last().cloned() {
                    app.stack.push(object);
                }
            }
        }
        Message::Quit => app.should_quit = true,
        Message::Evaluate => {
            if let Some(Expression(expression)) = app.stack.pop() {
                if let Ok(result) = expression.evaluate() {
                    app.stack.push(result);
                } else {
                    panic!()
                }
            }
        }
    }
}
