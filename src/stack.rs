pub struct Stack(Vec<f64>);

impl Stack {
    pub fn push(&mut self, num: f64) {
        self.0.push(num);
    }
    pub fn pop(&mut self) -> Option<f64> {
        self.0.pop()
    }
    pub fn len(&mut self) -> usize {
        self.0.len()
    }
}
