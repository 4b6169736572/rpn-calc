//! The error type for parsing and evaluation.

/// What can go wrong while parsing input or evaluating an expression.
#[derive(Debug)]
pub enum Error {
    /// A parsing or stack problem, with a short message.
    /// [`Input::parse`](crate::app::Input::parse) returns it for text that isn't a
    /// number.
    StackError(&'static str),
    /// A division by zero, returned by
    /// [`Expression::evaluate`](crate::app::Expression::evaluate).
    DivByZero,
}
