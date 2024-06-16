use super::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    vec::Vec,
};

struct Internal_user_type {
    pub Name: String,
}

struct Internal_group_type {
    pub Name: String,
    pub Users: HashSet<User_identifier_type>,
}

pub struct Manager_type {
    Users: Arc<RwLock<HashMap<User_identifier_type, Internal_user_type>>>,
    Groups: Arc<RwLock<HashMap<Group_identifier_type, Internal_group_type>>>,
}

impl Manager_type {
    const Root_user_identifier: User_identifier_type = 0;

    pub fn New() -> Self {
        Self {
            Users: Arc::new(RwLock::new(HashMap::new())),
            Groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn Get_new_group_identifier(&self) -> Group_identifier_type {
        let Groups = self.Groups.read().unwrap();

        let mut Group_identifier: Group_identifier_type = 0;
        while Groups.contains_key(&Group_identifier) {
            if Group_identifier == Group_identifier_type::MAX {
                panic!("No remaining group identifier !");
            }
            Group_identifier += 1;
        }
        Group_identifier
    }

    pub fn Get_new_user_identifier(&self) -> User_identifier_type {
        let Users = self.Users.read().unwrap();

        let mut User_identifier: User_identifier_type = 0;
        while Users.contains_key(&User_identifier) {
            if User_identifier == User_identifier_type::MAX {
                panic!("No remaining group identifier !");
            }
            User_identifier += 1;
        }
        User_identifier
    }

    pub fn Create_group(
        &self,
        Name: &str,
        Identifier: Option<Group_identifier_type>,
    ) -> Result<Group_identifier_type, ()> {
        let Identifier = match Identifier {
            Some(Identifier) => Identifier,
            None => self.Get_new_group_identifier(),
        };

        let mut Groups = self.Groups.write().unwrap();

        let Group = Internal_group_type {
            Name: Name.to_string(),
            Users: HashSet::new(),
        };

        if self.Exists_group(Identifier) {
            return Err(());
        }

        if let Some(_) = Groups.insert(Identifier, Group) {
            return Err(());
        }
        Ok(Identifier)
    }

    pub fn Is_root(&self, Identifier: User_identifier_type) -> bool {
        Self::Root_user_identifier == Identifier
    }

    pub fn Exists_group(&self, Identifier: Group_identifier_type) -> bool {
        self.Groups.read().unwrap().contains_key(&Identifier)
    }

    pub fn Exists_user(&self, Identifier: User_identifier_type) -> bool {
        self.Users.read().unwrap().contains_key(&Identifier)
    }

    pub fn Add_to_group(
        &self,
        User_identifier: User_identifier_type,
        Group_identifier: Group_identifier_type,
    ) -> Result<(), ()> {
        let mut Groups = self.Groups.write().unwrap();
        if !self.Exists_group(Group_identifier) {
            return Err(());
        }
        if !Groups
            .get_mut(&Group_identifier)
            .unwrap()
            .Users
            .insert(User_identifier)
        {
            return Err(());
        }
        Ok(())
    }

    pub fn Get_group_name(&self, Identifier: Group_identifier_type) -> Option<String> {
        let Groups = self.Users.read().unwrap();
        Some(Groups.get(&Identifier).unwrap().Name.clone())
    }

    pub fn Get_group_users<'a>(
        &'a self,
        Identifier: Group_identifier_type,
    ) -> Option<Vec<User_identifier_type>> {
        let Groups = self.Groups.read().unwrap();
        Some(
            Groups
                .get(&Identifier)
                .unwrap()
                .Users
                .clone()
                .into_iter()
                .collect(),
        )
    }

    pub fn Get_user_name(&self, Identifier: User_identifier_type) -> Option<String> {
        let Users = self.Users.read().unwrap();
        Some(Users.get(&Identifier).unwrap().Name.clone())
    }

    pub fn Check_credentials(&self, User_name: &str, Password: &str) -> bool {
        true
    }
}
