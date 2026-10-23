//! Calculator state and behaviour: the [`App`] state, the objects on the stack,
//! the key mapping in [`handle_key`] and the state changes in [`update`].

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

/// The whole calculator state: what [`update`] changes and [`crate::ui::view`]
/// draws.
#[derive(Default)]
pub struct App {
    /// The stack, oldest item first. The last element is the top (level 1), drawn
    /// at the bottom of the Stack panel.
    pub stack: Vec<StackObject>,
    /// Text typed on the Input line that hasn't been pushed yet.
    pub input: Input,
    /// Set by [`Message::Quit`]; [`run`] stops when it is `true`.
    pub should_quit: bool,
    /// Stored variables by their one-letter name. A value can be a number or a
    /// whole expression.
    pub variables: HashMap<char, StackObject>,
    /// The current input mode, which decides how [`handle_key`] reads the next key.
    pub mode: Mode,
    /// TODO
    pub history: Vec<Action>,
}

impl App {
    /// Parses the Input line as a number, pushes it, and clears the input.
    ///
    /// # Panics
    ///
    /// Panics if the input is not a valid `f64`, such as `1.2.3` or a lone `.`.
    pub fn push_input(&mut self) {
        match self.input.parse() {
            Ok(object) => self.stack.push(object),
            Err(errror) => panic!(),
        }
    }
}

/// How [`handle_key`] reads the next key press.
///
/// [`update`] returns to [`Mode::Normal`] after a store, recall, symbol push or
/// swap, and after a repeat count has been used.
#[derive(Default, Clone)]
pub enum Mode {
    /// Numbers, operators and commands; see [`handle_key`] for the keys.
    #[default]
    Normal,
    /// After `s`: the next letter names the variable to store into.
    Store,
    /// After `r`: the next letter names the variable to recall.
    Recall,
    /// After `'`: the next letter is pushed as a variable symbol.
    Alpha,
    /// A repeat count is set. Keys work as in [`Mode::Normal`], and the next
    /// `Enter`, `Backspace` or `Tab` uses the count.
    Repeat(u16),
    /// After `Ctrl+N`: the next digit becomes the repeat count.
    IntoRepeat,
}

/// TODO
pub struct Action {
    message: Message,
    input: Option<Input>,
}

/// The text typed on the Input line, not yet pushed onto the stack.
#[derive(Clone, Default)]
pub struct Input(String);
impl Input {
    /// Appends a typed character.
    fn push(&mut self, character: char) {
        self.0.push(character);
    }
    /// Removes the last character.
    fn pop(&mut self) {
        self.0.pop();
    }
    /// Parses the text as an `f64`. On success, clears the input and returns the
    /// number as a [`StackObject::Number`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::StackError`] with `"Parsing Error"` if the text is not a
    /// valid number; the input is left as it was.
    fn parse(&mut self) -> Result<StackObject, Error> {
        if let Ok(num) = self.0.parse::<f64>() {
            self.0.clear();
            Ok(Number(num))
        } else {
            Err(Error::StackError("Parsing Error"))
        }
    }
    /// Returns `true` if nothing has been typed.
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns a copy of the typed text, for display.
    pub fn string(&self) -> String {
        self.0.clone()
    }
}

/// One item on the stack.
#[derive(Clone)]
pub enum StackObject {
    /// A plain number.
    Number(f64),
    /// An operation that hasn't been calculated yet.
    Expression(Expression),
    /// A variable name or a bracket.
    Symbol(Symbol),
}

impl StackObject {
    /// Formats the object as one line of text for the Stack panel.
    ///
    /// Numbers use `f64`'s `Display` (`3`, `3.5`) and symbols their character.
    /// Expressions use infix notation. `depth` is the nesting level: depth 0 is
    /// printed bare and every nested expression gets parentheses, as in
    /// `(3 + 4) * 2`.
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
    /// Resolves the object for use inside an evaluation.
    ///
    /// An expression is calculated with [`Expression::evaluate`], and a variable
    /// with a stored value becomes that value. Numbers, variables without a value
    /// and brackets come back unchanged.
    ///
    /// # Panics
    ///
    /// Panics if calculating the expression returns an error, such as a division
    /// by zero.
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

/// Converts a stack object to its display text, as [`StackObject::render`]
/// does at depth 0.
impl From<StackObject> for String {
    fn from(object: StackObject) -> Self {
        object.render(0)
    }
}

/// A binary operation that hasn't been calculated yet. Operands can be numbers,
/// variables or other expressions, so expressions form a tree.
///
/// [`update`] builds one when an operator key is pressed: the second item
/// becomes `left` and the top item `right`. [`Expression::evaluate`] calculates
/// it, normally after `=`.
#[derive(Clone)]
pub struct Expression {
    /// The first operand: the item that was second from the top.
    left: Box<StackObject>,
    /// The second operand: the item that was on top.
    right: Box<StackObject>,
    /// The operation that joins them.
    function: Box<BinaryFunction>,
}

use BinaryFunction::*;
use StackObject::*;

impl Expression {
    /// Calculates the expression.
    ///
    /// If both operands are numbers, `function` is applied to them. Otherwise each
    /// operand is resolved with [`StackObject::evaluate`], so sub-expressions are
    /// calculated and variables replaced by their values, and the result is
    /// evaluated again.
    ///
    /// Every variable in the tree needs a stored value. With an unbound variable
    /// the recursion doesn't end, and the program aborts with a stack overflow.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DivByZero`] if both operands are numbers, `function` is
    /// [`BinaryFunction::Div`] and `right` is `0`.
    ///
    /// # Panics
    ///
    /// Panics instead of returning [`Error::DivByZero`] when the zero divisor only
    /// shows up while operands are resolved, as in `(1 / 0) + 2`, `1 / (2 - 2)` or
    /// `1 / x` with `x` = 0.
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

/// A stack item that isn't a number or an expression.
#[derive(Clone)]
pub enum Symbol {
    /// A variable named by one letter, pushed with `'` and the letter. Its value,
    /// if any, is in [`App::variables`].
    Variable(char),
    /// An opening bracket.
    Left(GroupingSymbol),
    /// A closing bracket.
    Right(GroupingSymbol),
}

impl Symbol {
    /// Returns a variable's stored value if it has one; otherwise returns the
    /// symbol unchanged, as a [`StackObject::Symbol`].
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
    /// Returns the symbol's text: the variable's letter or the bracket.
    pub fn render(&self) -> String {
        match self {
            Symbol::Variable(char) => char.to_string(),
            Symbol::Left(grouper) => grouper.left(),
            Symbol::Right(grouper) => grouper.right(),
        }
    }
}

/// The bracket kinds a [`Symbol::Left`] or [`Symbol::Right`] can have.
#[derive(Clone)]
pub enum GroupingSymbol {
    /// `(` and `)`
    Paren,
    /// `[` and `]`
    Bracket,
    /// `<` and `>`
    Angle,
    /// `{` and `}`
    Curly,
}

impl GroupingSymbol {
    /// Returns the opening character, such as `(`.
    pub fn left(&self) -> String {
        match self {
            GroupingSymbol::Paren => String::from("("),
            GroupingSymbol::Bracket => String::from("["),
            GroupingSymbol::Angle => String::from("<"),
            GroupingSymbol::Curly => String::from("{"),
        }
    }
    /// Returns the closing character, such as `)`.
    pub fn right(&self) -> String {
        match self {
            GroupingSymbol::Paren => String::from(")"),
            GroupingSymbol::Bracket => String::from("]"),
            GroupingSymbol::Angle => String::from(">"),
            GroupingSymbol::Curly => String::from("}"),
        }
    }
}

/// An operation on two operands, chosen with the operator keys.
#[derive(Clone)]
pub enum BinaryFunction {
    /// `+`: `left + right`
    Add,
    /// `-`: `left - right`
    Sub,
    /// `/`: `left / right`
    Div,
    /// `*`: `left * right`
    Mult,
    /// `^`: `left` to the power of `right`, using [`f64::powf`]
    Exp,
}

impl BinaryFunction {
    /// Joins two rendered operands with the operator, as `a + b`, `a - b`, `a / b`,
    /// `a * b`, or `a^b` without spaces.
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

/// Everything a key press can ask [`update`] to do. [`handle_key`] creates
/// them; each variant says what [`update`] does with it.
#[derive(Clone)]
pub enum Message {
    /// `Tab`: swap the top two items. With a repeat count `n` and more than `n`
    /// items on the stack, move the item at level `n` to the top instead.
    Flip,
    /// A digit or `.`: append it to the Input line.
    Numerical(char),
    /// An operator key: push any typed number, then join the top two items into an
    /// [`Expression`]. With fewer than two items, nothing is joined.
    Function(BinaryFunction),
    /// `Enter`: push the typed number, or duplicate the top item if nothing is
    /// typed. With a repeat count `n` of 1 or more, `n` copies are added in total.
    Return,
    /// `Backspace`: drop the top item. With a repeat count `n`, push `n - 1` copies
    /// of the top item instead.
    Backspace,
    /// `=`: pop the top item. An expression is replaced by its result and a
    /// variable by its stored value. Anything else, including a number or a
    /// variable without a value, is removed.
    Evaluate,
    /// `q` or `Ctrl+C`: stop the program.
    Quit,
    /// `s`, `r`, `'` or `Esc`: switch to [`Mode::Store`], [`Mode::Recall`],
    /// [`Mode::Alpha`] or [`Mode::Normal`].
    ChangeMode(Mode),
    /// A letter in [`Mode::Store`]: save the typed number under that name, or a
    /// copy of the top item if nothing is typed. A typed number is not pushed.
    Store(char),
    /// A letter in [`Mode::Recall`]: push the variable's stored value, if it has
    /// one.
    Recall(char),
    /// A letter in [`Mode::Alpha`]: push it as a [`Symbol::Variable`].
    Alpha(char),
    /// A digit in [`Mode::IntoRepeat`]: switch to [`Mode::Repeat`] with that count.
    Repeat(u16),
    /// `Ctrl+N`: switch to [`Mode::IntoRepeat`] to wait for a count.
    IntoRepeat,
}

/// Runs the main loop until [`App::should_quit`] is set.
///
/// Each pass draws the screen with [`crate::ui::view`], waits for the next
/// terminal event, turns a key press into a [`Message`] with [`handle_key`], and
/// applies it with [`update`]. Other events, such as a resize, just lead to the
/// next redraw.
///
/// # Errors
///
/// Returns any I/O error from drawing or reading events.
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

/// Keys that type into the Input line: the digits and `.`. Also the keys
/// [`Mode::IntoRepeat`] accepts as a count.
const NUMERICAL: &str = "0123456789.";
/// Keys accepted as variable names: ASCII letters except `e` and `E`.
const CHARACTER: &str = "abcdfghijklmnopqrstuvwxyzABCDFGHIJKLMNOPQRSTUVWXYZ";

/// Maps a key event to a [`Message`] for the current [`Mode`], or `None` if the
/// key does nothing there.
///
/// Only key presses count; release and repeat events return `None`. `Ctrl+C`
/// (quit) and `Ctrl+N` (repeat count) work in every mode.
///
/// | Mode | Keys |
/// |------|------|
/// | [`Mode::Normal`], [`Mode::Repeat`] | digits and `.`, `+ - * / ^`, `=`, `Enter`, `Backspace`, `Tab`, `Esc`, `s`, `r`, `'`, `q` |
/// | [`Mode::Store`], [`Mode::Recall`], [`Mode::Alpha`] | a letter from [`CHARACTER`], `Backspace` |
/// | [`Mode::IntoRepeat`] | a digit |
///
/// # Panics
///
/// Panics if `.` is pressed in [`Mode::IntoRepeat`].
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

/// Applies a [`Message`] to the [`App`]. This is the only place the state
/// changes while the program runs; each [`Message`] variant describes its
/// effect.
///
/// # Panics
///
/// - [`Message::Function`], [`Message::Return`] or [`Message::Store`] with typed
///   text that isn't a number (see [`App::push_input`]).
/// - [`Message::Return`] with nothing typed, or [`Message::Backspace`], when a
///   repeat count of 2 or more is set and the stack is empty.
/// - [`Message::Evaluate`] on an expression that divides by zero.
pub fn update(app: &mut App, msg: Message) {
    match msg {
        Message::Numerical(c) => app.input.push(c),
        Message::Function(function) => {
            if app.stack.len() < 2 {
                return;
            }
            if !app.input.is_empty() {
                app.history.push(Action {
                    input: Some(app.input.clone()),
                    message: Message::Return,
                });
                app.push_input();
            }
            let (right, left) = (app.stack.pop(), app.stack.pop());
            if let (Some(left), Some(right)) = (left, right) {
                app.stack.push(Expression(Expression {
                    left: Box::new(left),
                    right: Box::new(right),
                    function: Box::new(function),
                }));
                app.history.push(Action {
                    input: None,
                    message: Message::Function(Div),
                });
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
