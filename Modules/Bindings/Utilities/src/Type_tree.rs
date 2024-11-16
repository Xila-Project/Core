use std::collections::HashMap;

use quote::ToTokens;
use syn::Path;

#[derive(Debug)]
pub struct Type_tree_type {
    Type_tree: HashMap<String, String>,
}

impl Default for Type_tree_type {
    fn default() -> Self {
        let Map = HashMap::new();

        Self { Type_tree: Map }
    }
}

impl Type_tree_type {
    pub fn Insert(&mut self, Type: String, Alias: String) {
        self.Type_tree.insert(Type, Alias);
    }

    pub fn Resolve(&self, Path: &Path) -> String {
        let Path_string = Path.to_token_stream().to_string();

        if let Some(Alias) = self.Type_tree.get(&Path_string) {
            Alias.clone()
        } else {
            Path_string
        }
    }
}
