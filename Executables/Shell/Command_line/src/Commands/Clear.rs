use crate::Shell_type;

impl Shell_type {
    pub async fn Clear(&mut self, _: &[&str]) {
        self.Standard.Print("\x1B[2J").await;
        self.Standard.Print("\x1B[H").await;
    }
}
