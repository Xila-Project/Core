use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeTree {
    type_tree: HashMap<String, String>,
}

impl Default for TypeTree {
    fn default() -> Self {
        let map = HashMap::new();

        Self { type_tree: map }
    }
}

impl TypeTree {
    pub fn insert(&mut self, r#type: String, alias: String) {
        self.type_tree.insert(r#type, alias);
    }
}
