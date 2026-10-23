//! A terminal calculator that uses Reverse Polish Notation, built with [ratatui].
//!
//! Numbers, unevaluated expressions and variable symbols share one stack of
//! [`app::StackObject`]s. Operators join the top two items into an
//! [`app::Expression`] instead of calculating at once; `=` evaluates the top item.
//!
//! The program follows The Elm Architecture. One [`app::App`] value holds all
//! state, and [`app::run`] repeats four steps until the user quits:
//!
//! ```text
//! 1. draw    ui::view(frame, &app)         render the current state
//! 2. read    event::read()                 wait for the next terminal event
//! 3. map     app::handle_key(key, &mode)   key press -> Option<Message>
//! 4. apply   app::update(&mut app, msg)    the only step that changes state
//! ```
//!
//! | Module     | Contents                                                  |
//! |------------|-----------------------------------------------------------|
//! | [`app`]    | State, stack objects, key mapping and the update step     |
//! | [`ui`]     | Layout and drawing of the Stack, Input and Help panels    |
//! | [`errors`] | [`errors::Error`], returned by parsing and evaluation     |
//! | [`domain`] | Commented-out arithmetic from the first working version   |
//! | [`stack`]  | Commented-out `Stack` type from the first working version |
#![allow(unused)]
mod app;
mod domain;
mod errors;
mod stack;
mod ui;

/// Starts the calculator.
///
/// [`ratatui::run`] switches the terminal to raw mode and the alternate screen,
/// passes it to [`app::run`], and restores it afterwards. It also restores the
/// terminal before a panic message is printed. If [`app::run`] returns an I/O
/// error, `main` panics.
fn main() {
    ratatui::run(app::run).unwrap();
}
