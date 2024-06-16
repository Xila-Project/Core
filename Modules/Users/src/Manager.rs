use super::*;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

struct Internal_user_type {
    pub Name: String,
}

pub struct Manager_type {
    Users: Arc<RwLock<HashMap<User_identifier_type, Internal_user_type>>>,
}

impl Manager_type {
    const Root_user_identifier: User_identifier_type = 0;

    pub fn New() -> Self {
        Self {
            Users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn Get_user_name(&self, Identifier: User_identifier_type) -> Option<String> {
        let Users = self.Users.read().unwrap();
        Some(Users.get(&Identifier).unwrap().Name.clone())
    }

    pub fn Check_credentials(&self, User_name: &str, Password: &str) -> bool {
        true
    }
}
