use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
    vec::Vec,
};
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
        let mut Groups = BTreeMap::new();
        Groups.insert(
            Group_identifier_type::Root,
            Internal_group_type {
                Name: "Root".to_string(),
                Users: BTreeSet::new(),
            },
        );

        let mut Users = BTreeMap::new();
        Users.insert(
            User_identifier_type::Root,
            Internal_user_type {
                Name: "Root".to_string(),
                Primary_group: Group_identifier_type::Root,
            },
        );

        Self(RwLock::new(Internal_manager_type { Users, Groups }))
    }

    pub fn Get_new_group_identifier(&self) -> Result_type<Group_identifier_type> {
        let Inner = self.0.read()?;

        let mut Identifier = Group_identifier_type::Minimum;

        while Inner.Groups.contains_key(&Identifier) {
            Identifier += 1;

            if Identifier == Group_identifier_type::Maximum {
                return Err(Error_type::Too_many_groups);
            }
        }

        Ok(Identifier)
    }

    pub fn Get_new_user_identifier(&self) -> Result_type<User_identifier_type> {
        let Inner = self.0.read()?;

        let mut Identifier = User_identifier_type::Minimum;

        while Inner.Users.contains_key(&Identifier) {
            Identifier += 1;

            if Identifier == User_identifier_type::Maximum {
                return Err(Error_type::Too_many_users);
            }
        }

        Ok(Identifier)
    }

    pub fn Add_user(
        &self,
        Identifier: User_identifier_type,
        Name: &str,
        Primary_group: Group_identifier_type,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        // - Check if user identifier is unique
        if Inner.Users.contains_key(&Identifier) {
            return Err(Error_type::Duplicate_user_identifier);
        }

        // - Check if user name is unique
        if Inner.Users.values().any(|User| User.Name == Name) {
            return Err(Error_type::Duplicate_user_name);
        }

        // - Add user to the users map
        let User = Internal_user_type {
            Name: Name.to_string(),
            Primary_group,
        };

        if Inner.Users.insert(Identifier, User).is_some() {
            return Err(Error_type::Duplicate_user_identifier); // Shouldn't happen
        }

        // - Add user to the primary group
        if !Inner
            .Groups
            .get_mut(&Primary_group)
            .ok_or(Error_type::Invalid_group_identifier)?
            .Users
            .insert(Identifier)
        {
            return Err(Error_type::Duplicate_user_identifier); // Shouldn't happen
        }

        Ok(())
    }

    pub fn Add_group(
        &self,
        Identifier: Group_identifier_type,
        Name: &str,
        Users: &[User_identifier_type],
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        // - Check if group identifier is unique
        if Inner.Groups.contains_key(&Identifier) {
            return Err(Error_type::Duplicate_group_identifier);
        }

        // - Check if group name is unique
        if Inner.Groups.values().any(|Group| Group.Name == Name) {
            return Err(Error_type::Duplicate_group_name);
        }

        let Group = Internal_group_type {
            Name: Name.to_string(),
            Users: BTreeSet::from_iter(Users.iter().cloned()),
        };

        if Inner.Groups.insert(Identifier, Group).is_some() {
            return Err(Error_type::Duplicate_group_identifier); // Shouldn't happen
        }

        Ok(())
    }

    pub fn Is_root(Identifier: User_identifier_type) -> bool {
        User_identifier_type::Root == Identifier
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
    ) -> Result_type<BTreeSet<Group_identifier_type>> {
        let Inner = self.0.read()?;

        let mut User_groups: BTreeSet<Group_identifier_type> = BTreeSet::new();

        User_groups.extend(
            Inner
                .Groups
                .iter()
                .filter(|(_, Group)| Group.Users.contains(&Identifier))
                .map(|(Identifier, _)| *Identifier),
        );

        Ok(User_groups)
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

    pub fn Get_user_identifier(&self, Name: &str) -> Result_type<User_identifier_type> {
        Ok(*self
            .0
            .read()?
            .Users
            .iter()
            .find(|(_, User)| User.Name == Name)
            .ok_or(Error_type::Invalid_user_identifier)?
            .0)
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
    fn Create_user() {
        let Manager = Manager_type::New();
        let User_name = "Alice";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::Root)
            .unwrap();
        assert!(Manager.Exists_user(Identifier).unwrap());
    }

    #[test]
    fn Create_user_duplicate() {
        let User_name_1 = "Alice";
        let User_name_2 = "Bob";

        let Identifier_1 = User_identifier_type::New(1000);
        let Identifier_2 = User_identifier_type::New(1001);

        // Same identifier
        let Manager = Manager_type::New();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::Root)
            .unwrap();

        let Result = Manager.Add_user(Identifier_1, User_name_2, Group_identifier_type::Root);
        assert_eq!(Result, Err(Error_type::Duplicate_user_identifier));

        // Same name
        let Manager = Manager_type::New();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::Root)
            .unwrap();

        let Result = Manager.Add_user(Identifier_2, User_name_1, Group_identifier_type::Root);
        assert_eq!(Result, Err(Error_type::Duplicate_user_name));

        // Same name and identifier
        let Manager = Manager_type::New();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::Root)
            .unwrap();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::Root)
            .unwrap_err();
    }

    #[test]
    fn Create_group() {
        let Manager = Manager_type::New();
        let Group_name = "Developers";
        let Group_id = Group_identifier_type::New(1000);
        let Result = Manager.Add_group(Group_id, Group_name, &[]);
        assert!(Result.is_ok());
        assert!(Manager.Exists_group(Group_id).unwrap());
    }

    #[test]
    fn Create_group_duplicate() {
        let Group_name_1 = "Developers";
        let Group_name_2 = "Testers";

        let Group_id_1 = Group_identifier_type::New(1000);
        let Group_id_2 = Group_identifier_type::New(1001);

        // Same identifier
        let Manager = Manager_type::New();

        Manager.Add_group(Group_id_1, Group_name_1, &[]).unwrap();

        let Result = Manager.Add_group(Group_id_1, Group_name_2, &[]);
        assert_eq!(Result, Err(Error_type::Duplicate_group_identifier));

        // Same name
        let Manager = Manager_type::New();

        Manager.Add_group(Group_id_1, Group_name_1, &[]).unwrap();

        let Result = Manager.Add_group(Group_id_2, Group_name_1, &[]);
        assert_eq!(Result, Err(Error_type::Duplicate_group_name));

        // Same name and identifier
        let Manager = Manager_type::New();

        Manager.Add_group(Group_id_1, Group_name_1, &[]).unwrap();

        Manager
            .Add_group(Group_id_1, Group_name_1, &[])
            .unwrap_err();
    }

    #[test]
    fn Is_root() {
        let Root_id = User_identifier_type::Root;
        assert!(Manager_type::Is_root(Root_id));
    }

    #[test]
    fn Is_in_group() {
        let Manager = Manager_type::New();
        let User_name = "Bob";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::Root)
            .unwrap();
        let Group_name = "Admins";
        let Group_id = Group_identifier_type::New(1000);

        Manager.Add_group(Group_id, Group_name, &[]).unwrap();
        Manager.Add_to_group(Identifier, Group_id).unwrap();
        assert!(Manager.Is_in_group(Identifier, Group_id));
    }

    #[test]
    fn Get_user_groups() {
        let Manager = Manager_type::New();

        let User_name = "Charlie";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::Root)
            .unwrap();
        let Group_name1 = "TeamA";
        let Group_id1 = Group_identifier_type::New(1000);

        Manager.Add_group(Group_id1, Group_name1, &[]).unwrap();
        let Group_name2 = "TeamB";
        let Group_id2 = Group_identifier_type::New(1001);

        Manager.Add_group(Group_id2, Group_name2, &[]).unwrap();
        Manager.Add_to_group(Identifier, Group_id1).unwrap();
        Manager.Add_to_group(Identifier, Group_id2).unwrap();
        let Groups = Manager.Get_user_groups(Identifier).unwrap();
        println!("{:?}", Groups);
        assert_eq!(Groups.len(), 3);
        assert!(
            Groups.contains(&Group_id1)
                && Groups.contains(&Group_id2)
                && Groups.contains(&Group_identifier_type::Root)
        );
    }

    #[test]
    fn Get_group_name() {
        let Manager = Manager_type::New();
        let Group_name = "QA";
        let Group_id = Group_identifier_type::New(1000);
        Manager.Add_group(Group_id, Group_name, &[]).unwrap();
        let Retrieved_name = Manager.Get_group_name(Group_id).unwrap();
        assert_eq!(Group_name, Retrieved_name);
    }

    #[test]
    fn Get_group_users() {
        let Manager = Manager_type::New();
        let User_name = "Dave";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::Root)
            .unwrap();
        let Group_name = "Engineers";
        let Group_id = Group_identifier_type::New(1000);
        Manager.Add_group(Group_id, Group_name, &[]).unwrap();
        Manager.Add_to_group(Identifier, Group_id).unwrap();
        let Users = Manager.Get_group_users(Group_id).unwrap();
        assert_eq!(Users.len(), 1);
        assert!(Users.contains(&Identifier));
    }

    #[test]
    fn Get_user_name() {
        let Manager = Manager_type::New();
        let User_name = "Eve";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::Root)
            .unwrap();
        let Retrieved_name = Manager.Get_user_name(Identifier).unwrap();
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
