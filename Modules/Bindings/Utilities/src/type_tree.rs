use std::collections::HashMap;

use quote::ToTokens;
use syn::Path;

#[derive(Debug)]
pub struct Type_tree_type {
    type_tree: HashMap<String, String>,
}

impl Default for Type_tree_type {
    fn default() -> Self {
        let map = HashMap::new();

        Self { type_tree: map }
    }
}

impl Type_tree_type {
    pub fn insert(&mut self, r#type: String, alias: String) {
        self.type_tree.insert(r#type, alias);
    }

    pub fn resolve(&self, path: &Path) -> String {
        let path_string = path.to_token_stream().to_string();

        if let Some(alias) = self.type_tree.get(&path_string) {
            alias.clone()
        } else {
            path_string
        }
    }
}
