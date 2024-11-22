use crate::Shell_type;

impl Shell_type {
    pub fn Clear(&mut self, _: &[&str]) {
        self.Standard.Print("\x1B[2J");
        self.Standard.Print("\x1B[H");
    }
}
