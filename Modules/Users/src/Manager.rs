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

struct Internal_manager_type {
    pub Users: HashMap<User_identifier_type, Internal_user_type>,
    pub Groups: HashMap<Group_identifier_type, Internal_group_type>,
}

#[derive(Clone)]
pub struct Manager_type(Arc<RwLock<Internal_manager_type>>);

impl Manager_type {
    pub fn New() -> Self {
        Self(Arc::new(RwLock::new(Internal_manager_type {
            Users: HashMap::new(),
            Groups: HashMap::new(),
        })))
    }

    fn Get_new_group_identifier(&self) -> Option<Group_identifier_type> {
        let Inner = self.0.read().ok()?;

        (0..Group_identifier_type::MAX).find(|Identifier| !Inner.Groups.contains_key(Identifier))
    }

    fn Get_new_user_identifier(&self) -> Option<User_identifier_type> {
        let Inner = self.0.read().ok()?;

        (0..User_identifier_type::MAX).find(|Identifier| !Inner.Users.contains_key(Identifier))
    }

    pub fn Create_user(&self, Name: &str) -> Result<User_identifier_type> {
        let Identifier = match self.Get_new_user_identifier() {
            Some(Identifier) => Identifier,
            None => return Err(Error_type::Too_many_users),
        };

        let User = Internal_user_type {
            Name: Name.to_string(),
        };

        if self.Exists_user(Identifier)? {
            return Err(Error_type::Duplicate_user_identifier);
        }

        let mut Inner = self.0.write().unwrap();

        if Inner.Users.insert(Identifier, User).is_some() {
            return Err(Error_type::Duplicate_user_identifier); // Shouldn't happen
        }
        Ok(Identifier)
    }

    pub fn Create_group(
        &self,
        Name: &str,
        Identifier: Option<Group_identifier_type>,
    ) -> Result<Group_identifier_type> {
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
            return Err(Error_type::Duplicate_group_identifier);
        }

        if Groups.insert(Identifier, Group).is_some() {
            return Err(Error_type::Duplicate_group_identifier); // Shouldn't happen
        }
        Ok(Identifier)
    }

    pub fn Is_root(Identifier: User_identifier_type) -> bool {
        crate::Root_user_identifier == Identifier
    }

    pub fn Is_in_group(
        &self,
        User_identifier: User_identifier_type,
        Group_identifier: Group_identifier_type,
    ) -> bool {
        let Inner = self.0.read().unwrap();
        Inner
            .Groups
            .get(&Group_identifier)
            .unwrap()
            .Users
            .contains(&User_identifier)
    }

    pub fn Get_user_groups(
        &self,
        Identifier: User_identifier_type,
    ) -> Option<Vec<Group_identifier_type>> {
        let Inner = self.0.read().unwrap();
        Some(
            Inner
                .Groups
                .iter()
                .filter(|(_, Group)| Group.Users.contains(&Identifier))
                .map(|(Identifier, _)| *Identifier)
                .collect(),
        )
    }

    pub fn Exists_group(&self, Identifier: Group_identifier_type) -> Result<bool> {
        Ok(self.0.read()?.Groups.contains_key(&Identifier))
    }

    pub fn Exists_user(&self, Identifier: User_identifier_type) -> Result<bool> {
        self.Users.read().unwrap().contains_key(&Identifier)
    }

    pub fn Add_to_group(
        &self,
        User_identifier: User_identifier_type,
        Group_identifier: Group_identifier_type,
    ) -> Result<()> {
        }
        if !Groups
            .get_mut(&Group_identifier)
            .unwrap()
            .Users
            .insert(User_identifier)
        {
            return Err(Error_type::Duplicate_group_identifier);
        }
        Ok(())
    }

    pub fn Get_group_name(&self, Identifier: Group_identifier_type) -> Result<String> {
    }

    pub fn Get_group_users(
        &self,
        Identifier: Group_identifier_type,
    ) -> Result<Vec<User_identifier_type>> {
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

    pub fn Get_user_name(&self, Identifier: User_identifier_type) -> Result<String> {
        Ok(self
            .0
            .read()?
            .Users
            .get(&Identifier)
            .ok_or(Error_type::Invalid_user_identifier)?
            .Name
            .clone())
    }

    pub fn Check_credentials(&self, _User_name: &str, _Password: &str) -> bool {
        true
    }
}
