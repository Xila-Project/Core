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
    pub fn insert(&mut self, Type: String, Alias: String) {
        self.type_tree.insert(Type, Alias);
    }

    pub fn Resolve(&self, Path: &Path) -> String {
        let path_string = Path.to_token_stream().to_string();

        if let Some(Alias) = self.type_tree.get(&path_string) {
            Alias.clone()
        } else {
            path_string
        }
    }
}
