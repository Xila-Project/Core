use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
    vec::Vec,
};

static mut Manager_instance: Option<Manager_type> = None;

pub fn Initialize() -> Result_type<&'static Manager_type> {
    if Is_initialized() {
        return Err(Error_type::Already_initialized);
    }

    unsafe {
        Manager_instance = Some(Manager_type::New());
    }

    Get_instance()
}

pub fn Get_instance() -> Result_type<&'static Manager_type> {
    unsafe { Manager_instance.as_ref().ok_or(Error_type::Not_initialized) }
}

pub fn Is_initialized() -> bool {
    unsafe { Manager_instance.is_some() }
}

struct Internal_user_type {
    pub Name: String,
    pub Primary_group: Group_identifier_type,
}

struct Internal_group_type {
    pub Name: String,
    pub Users: BTreeSet<User_identifier_type>,
}

struct Internal_manager_type {
    pub Users: BTreeMap<User_identifier_type, Internal_user_type>,
    pub Groups: BTreeMap<Group_identifier_type, Internal_group_type>,
}

pub struct Manager_type(RwLock<Internal_manager_type>);

impl Manager_type {
    fn New() -> Self {
        Self(RwLock::new(Internal_manager_type {
            Users: BTreeMap::new(),
            Groups: BTreeMap::new(),
        }))
    }

    fn Get_new_group_identifier(&self) -> Option<Group_identifier_type> {
        let Inner = self.0.read().ok()?;

        (0..Group_identifier_type::MAX).find(|Identifier| !Inner.Groups.contains_key(Identifier))
    }

    fn Get_new_user_identifier(&self) -> Option<User_identifier_type> {
        let Inner = self.0.read().ok()?;

        (0..User_identifier_type::MAX).find(|Identifier| !Inner.Users.contains_key(Identifier))
    }

    pub fn Create_user(
        &self,
        Name: &str,
        Primary_group: Group_identifier_type,
    ) -> Result_type<User_identifier_type> {
        let Identifier = match self.Get_new_user_identifier() {
            Some(Identifier) => Identifier,
            None => return Err(Error_type::Too_many_users),
        };

        let User = Internal_user_type {
            Name: Name.to_string(),
            Primary_group,
        };

        if self.Exists_user(Identifier)? {
            return Err(Error_type::Duplicate_user_identifier);
        }

        let mut Inner = self.0.write().unwrap();

        if Inner.Users.insert(Identifier, User).is_some() {
            return Err(Error_type::Duplicate_user_identifier); // Shouldn't happen
        }

        self.Add_to_group(Identifier, Primary_group)?;

        Ok(Identifier)
    }

    pub fn Create_group(
        &self,
        Name: &str,
        Identifier: Option<Group_identifier_type>,
    ) -> Result_type<Group_identifier_type> {
        let Identifier = match Identifier {
            Some(Identifier) => Identifier,
            None => self
                .Get_new_group_identifier()
                .ok_or(Error_type::Too_many_groups)?,
        };

        let Group = Internal_group_type {
            Name: Name.to_string(),
            Users: BTreeSet::new(),
        };

        if self.Exists_group(Identifier)? {
            return Err(Error_type::Duplicate_group_identifier);
        }

        let mut Inner = self.0.write().unwrap();

        if Inner.Groups.insert(Identifier, Group).is_some() {
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

        let mut Size = 1;

        Size += Inner
            .Groups
            .iter()
            .filter(|(_, Group)| Group.Users.contains(&Identifier))
            .count();

        let mut User_groups: Vec<Group_identifier_type> = Vec::with_capacity(Size);

        User_groups.push(Inner.Users.get(&Identifier).unwrap().Primary_group);

        User_groups.extend(
            Inner
                .Groups
                .iter()
                .filter(|(_, Group)| Group.Users.contains(&Identifier))
                .map(|(Identifier, _)| *Identifier),
        );

        Some(User_groups)
    }

    pub fn Exists_group(&self, Identifier: Group_identifier_type) -> Result_type<bool> {
        Ok(self.0.read()?.Groups.contains_key(&Identifier))
    }

    pub fn Exists_user(&self, Identifier: User_identifier_type) -> Result_type<bool> {
        Ok(self.0.read()?.Users.contains_key(&Identifier))
    }

    pub fn Add_to_group(
        &self,
        User_identifier: User_identifier_type,
        Group_identifier: Group_identifier_type,
    ) -> Result_type<()> {
        if !self.Exists_group(Group_identifier)? {
            return Err(Error_type::Invalid_group_identifier);
        }
        let mut Inner = self.0.write()?;
        if !Inner
            .Groups
            .get_mut(&Group_identifier)
            .unwrap()
            .Users
            .insert(User_identifier)
        {
            return Err(Error_type::Duplicate_group_identifier);
        }
        Ok(())
    }

    pub fn Get_group_name(&self, Identifier: Group_identifier_type) -> Result_type<String> {
        Ok(self.0.read()?.Groups.get(&Identifier).unwrap().Name.clone())
    }

    pub fn Get_group_users(
        &self,
        Identifier: Group_identifier_type,
    ) -> Result_type<Vec<User_identifier_type>> {
        Ok(self
            .0
            .read()?
            .Groups
            .get(&Identifier)
            .ok_or(Error_type::Invalid_group_identifier)?
            .Users
            .clone()
            .into_iter()
            .collect())
    }

    pub fn Get_user_name(&self, Identifier: User_identifier_type) -> Result_type<String> {
        Ok(self
            .0
            .read()?
            .Users
            .get(&Identifier)
            .ok_or(Error_type::Invalid_user_identifier)?
            .Name
            .clone())
    }

    pub fn Get_user_primary_group(
        &self,
        Identifier: User_identifier_type,
    ) -> Result_type<Group_identifier_type> {
        Ok(self
            .0
            .read()?
            .Users
            .get(&Identifier)
            .ok_or(Error_type::Invalid_user_identifier)?
            .Primary_group)
    }

    pub fn Check_credentials(&self, _User_name: &str, _Password: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn New() {
        let Manager = Manager_type::New();
        assert!(Manager.0.read().unwrap().Groups.is_empty());
    }

    #[test]
    fn Create_user() {
        let Manager = Manager_type::New();
        let User_name = "Alice";
        let Result = Manager.Create_user(User_name, Root_group_identifier);
        assert!(Result.is_ok());
        let User_id = Result.unwrap();
        assert!(Manager.Exists_user(User_id).unwrap());
    }

    #[test]
    fn Create_group() {
        let Manager = Manager_type::New();
        let Group_name = "Developers";
        let Result = Manager.Create_group(Group_name, None);
        assert!(Result.is_ok());
        let Group_id = Result.unwrap();
        assert!(Manager.Exists_group(Group_id).unwrap());
    }

    #[test]
    fn Is_root() {
        let Root_id = crate::Root_user_identifier;
        assert!(Manager_type::Is_root(Root_id));
    }

    #[test]
    fn Is_in_group() {
        let Manager = Manager_type::New();
        let User_name = "Bob";
        let User_id = Manager
            .Create_user(User_name, Root_group_identifier)
            .unwrap();
        let Group_name = "Admins";
        let Group_id = Manager.Create_group(Group_name, None).unwrap();
        Manager.Add_to_group(User_id, Group_id).unwrap();
        assert!(Manager.Is_in_group(User_id, Group_id));
    }

    #[test]
    fn Get_user_groups() {
        let Manager = Manager_type::New();

        let User_name = "Charlie";
        let User_id = Manager
            .Create_user(User_name, Root_group_identifier)
            .unwrap();
        let Group_name1 = "TeamA";
        let Group_id1 = Manager.Create_group(Group_name1, None).unwrap();
        let Group_name2 = "TeamB";
        let Group_id2 = Manager.Create_group(Group_name2, None).unwrap();
        Manager.Add_to_group(User_id, Group_id1).unwrap();
        Manager.Add_to_group(User_id, Group_id2).unwrap();
        let Groups = Manager.Get_user_groups(User_id).unwrap();
        assert_eq!(Groups.len(), 2);
        assert!(Groups.contains(&Group_id1) && Groups.contains(&Group_id2));
    }

    #[test]
    fn Get_group_name() {
        let Manager = Manager_type::New();
        let Group_name = "QA";
        let Group_id = Manager.Create_group(Group_name, None).unwrap();
        let Retrieved_name = Manager.Get_group_name(Group_id).unwrap();
        assert_eq!(Group_name, Retrieved_name);
    }

    #[test]
    fn Get_group_users() {
        let Manager = Manager_type::New();
        let User_name = "Dave";
        let User_id = Manager
            .Create_user(User_name, Root_group_identifier)
            .unwrap();
        let Group_name = "Engineers";
        let Group_id = Manager.Create_group(Group_name, None).unwrap();
        Manager.Add_to_group(User_id, Group_id).unwrap();
        let Users = Manager.Get_group_users(Group_id).unwrap();
        assert_eq!(Users.len(), 1);
        assert!(Users.contains(&User_id));
    }

    #[test]
    fn Get_user_name() {
        let Manager = Manager_type::New();
        let User_name = "Eve";
        let User_id = Manager
            .Create_user(User_name, Root_group_identifier)
            .unwrap();
        let Retrieved_name = Manager.Get_user_name(User_id).unwrap();
        assert_eq!(User_name, Retrieved_name);
    }

    #[test]
    fn Check_credentials() {
        let Manager = Manager_type::New();
        let User_name = "Frank";
        let Password = "password123";
        assert!(Manager.Check_credentials(User_name, Password));
    }
}
