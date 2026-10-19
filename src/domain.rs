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
