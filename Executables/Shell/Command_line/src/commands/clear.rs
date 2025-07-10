pub(crate) use crate::Shell;

impl Shell {
    pub async fn clear(&mut self, _: &[&str]) {
        self.standard.print("\x1B[2J").await;
        self.standard.print("\x1B[H").await;
    }
}
