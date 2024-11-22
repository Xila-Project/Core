use crate::Shell_type;

impl Shell_type {
    pub fn Echo(&mut self, Arguments: &[&str]) {
        for Argument in Arguments {
            self.Standard.Print(Argument);
            self.Standard.Print(" ");
        }
        self.Standard.Print("\n");
    }
}
