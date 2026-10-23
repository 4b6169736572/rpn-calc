//! Arithmetic from the first working version, in which the stack held plain
//! `f64` values: `add`, `sub`, `mult` and `div` each popped two numbers and
//! pushed the result. The whole file is commented out; operator keys now build
//! an [`Expression`](crate::app::Expression) instead.

// use crate::errors::{Error, Error::StackError};
// use crate::stack::Stack;
// 
// pub fn add(stack: &mut Stack) -> Result<f64, Error> {
//     if stack.len() < 2 {
//         return Err(StackError("Insufficient stack length."));
//     }
// 
//     let a = stack.pop().unwrap();
//     let b = stack.pop().unwrap();
// 
//     stack.push(a + b);
// 
//     Ok(a + b)
// }
// 
// pub fn sub(stack: &mut Stack) -> Result<f64, Error> {
//     if stack.len() < 2 {
//         return Err(StackError("Insufficient stack length."));
//     }
// 
//     let a = stack.pop().unwrap();
//     let b = stack.pop().unwrap();
// 
//     stack.push(a - b);
// 
//     Ok(a - b)
// }
// 
// pub fn mult(stack: &mut Stack) -> Result<f64, Error> {
//     if stack.len() < 2 {
//         return Err(StackError("Insufficient stack length."));
//     }
// 
//     let a = stack.pop().unwrap();
//     let b = stack.pop().unwrap();
// 
//     stack.push(a * b);
// 
//     Ok(a * b)
// }
// 
// pub fn div(stack: &mut Stack) -> Result<f64, Error> {
//     if stack.len() < 2 {
//         return Err(StackError("Insufficient stack length."));
//     }
// 
//     let bot = stack.pop().unwrap();
//     let top = stack.pop().unwrap();
// 
//     stack.push(top / bot);
// 
//     Ok(top / bot)
// }
