pub(crate) use crate::Shell_type;

impl Shell_type {
    pub async fn clear(&mut self, _: &[&str]) {
        self.standard.print("\x1B[2J").await;
        self.standard.print("\x1B[H").await;
    }
}
