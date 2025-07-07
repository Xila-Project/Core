use crate::Shell_type;

impl Shell_type {
    pub async fn clear(&mut self, _: &[&str]) {
        self.standard.Print("\x1B[2J").await;
        self.standard.Print("\x1B[H").await;
    }
}
