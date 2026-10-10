#[derive(Debug)]
pub enum Error {
    StackError(&'static str),
    DivByZero,
}
