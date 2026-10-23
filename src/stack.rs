//! The `Stack` type from the first working version, a wrapper around
//! `Vec<f64>` with `push`, `pop` and `len`. The whole file is commented out;
//! the stack is now [`App::stack`](crate::app::App::stack), a `Vec<StackObject>`.

// #[derive(Default, Debug, Clone)]
// pub struct Stack(Vec<f64>);
//
// impl Stack {
//     pub fn push(&mut self, num: f64) {
//         self.0.push(num);
//     }
//     pub fn pop(&mut self) -> Option<f64> {
//         self.0.pop()
//     }
//     pub fn len(&mut self) -> usize {
//         self.0.len()
//     }
// }
