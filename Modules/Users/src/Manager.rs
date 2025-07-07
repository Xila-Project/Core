use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec::Vec,
};
use Synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

use super::*;

static MANAGER_INSTANCE: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize() -> &'static Manager_type {
    MANAGER_INSTANCE.get_or_init(Manager_type::new)
}

pub fn Get_instance() -> &'static Manager_type {
    MANAGER_INSTANCE
        .try_get()
        .expect("User manager instance not initialized")
}

struct Internal_user_type {
    pub name: String,
    pub primary_group: Group_identifier_type,
}

struct Internal_group_type {
    pub name: String,
    pub users: BTreeSet<User_identifier_type>,
}

struct Internal_manager_type {
    pub users: BTreeMap<User_identifier_type, Internal_user_type>,
    pub groups: BTreeMap<Group_identifier_type, Internal_group_type>,
}

pub struct Manager_type(RwLock<CriticalSectionRawMutex, Internal_manager_type>);

impl Manager_type {
    fn new() -> Self {
        let mut groups = BTreeMap::new();
        groups.insert(
            Group_identifier_type::ROOT,
            Internal_group_type {
                name: "Root".to_string(),
                users: BTreeSet::new(),
            },
        );

        let mut Users = BTreeMap::new();
        Users.insert(
            User_identifier_type::ROOT,
            Internal_user_type {
                name: "Root".to_string(),
                primary_group: Group_identifier_type::ROOT,
            },
        );

        Self(RwLock::new(Internal_manager_type {
            users: Users,
            groups,
        }))
    }

    pub async fn Get_new_group_identifier(&self) -> Result_type<Group_identifier_type> {
        let inner = self.0.read().await;

        let mut Identifier = Group_identifier_type::MINIMUM;

        while inner.groups.contains_key(&Identifier) {
            Identifier += 1;

            if Identifier == Group_identifier_type::MAXIMUM {
                return Err(Error_type::Too_many_groups);
            }
        }

        Ok(Identifier)
    }

    pub async fn Get_new_user_identifier(&self) -> Result_type<User_identifier_type> {
        let inner = self.0.read().await;

        let mut Identifier = User_identifier_type::MINIMUM;

        while inner.users.contains_key(&Identifier) {
            Identifier += 1;

            if Identifier == User_identifier_type::MAXIMUM {
                return Err(Error_type::Too_many_users);
            }
        }

        Ok(Identifier)
    }

    pub async fn Add_user(
        &self,
        identifier: User_identifier_type,
        name: &str,
        primary_group: Group_identifier_type,
    ) -> Result_type<()> {
        let mut inner = self.0.write().await;

        // - Check if user identifier is unique
        if inner.users.contains_key(&identifier) {
            return Err(Error_type::Duplicate_user_identifier);
        }

        // - Check if user name is unique
        if inner.users.values().any(|User| User.name == name) {
            return Err(Error_type::Duplicate_user_name);
        }

        // - Add user to the users map
        let User = Internal_user_type {
            name: name.to_string(),
            primary_group,
        };

        if inner.users.insert(identifier, User).is_some() {
            return Err(Error_type::Duplicate_user_identifier); // Shouldn't happen
        }

        // - Add user to the primary group
        if !inner
            .groups
            .get_mut(&primary_group)
            .ok_or(Error_type::Invalid_group_identifier)?
            .users
            .insert(identifier)
        {
            return Err(Error_type::Duplicate_user_identifier); // Shouldn't happen
        }

        Ok(())
    }

    pub async fn Add_group(
        &self,
        identifier: Group_identifier_type,
        name: &str,
        users: &[User_identifier_type],
    ) -> Result_type<()> {
        let mut inner = self.0.write().await;

        // - Check if group identifier is unique
        if inner.groups.contains_key(&identifier) {
            return Err(Error_type::Duplicate_group_identifier);
        }

        // - Check if group name is unique
        if inner.groups.values().any(|Group| Group.name == name) {
            return Err(Error_type::Duplicate_group_name);
        }

        let Group = Internal_group_type {
            name: name.to_string(),
            users: BTreeSet::from_iter(users.iter().cloned()),
        };

        if inner.groups.insert(identifier, Group).is_some() {
            return Err(Error_type::Duplicate_group_identifier); // Shouldn't happen
        }

        Ok(())
    }

    pub fn Is_root(Identifier: User_identifier_type) -> bool {
        User_identifier_type::ROOT == Identifier
    }

    pub async fn Is_in_group(
        &self,
        user_identifier: User_identifier_type,
        group_identifier: Group_identifier_type,
    ) -> bool {
        let inner = self.0.read().await;
        inner
            .groups
            .get(&group_identifier)
            .unwrap()
            .users
            .contains(&user_identifier)
    }

    pub async fn Get_user_groups(
        &self,
        identifier: User_identifier_type,
    ) -> Result_type<BTreeSet<Group_identifier_type>> {
        let inner = self.0.read().await;

        let mut User_groups: BTreeSet<Group_identifier_type> = BTreeSet::new();

        User_groups.extend(
            inner
                .groups
                .iter()
                .filter(|(_, Group)| Group.users.contains(&identifier))
                .map(|(identifier, _)| *identifier),
        );

        Ok(User_groups)
    }

    pub async fn Exists_group(&self, Identifier: Group_identifier_type) -> Result_type<bool> {
        Ok(self.0.read().await.groups.contains_key(&Identifier))
    }

    pub async fn Exists_user(&self, Identifier: User_identifier_type) -> Result_type<bool> {
        Ok(self.0.read().await.users.contains_key(&Identifier))
    }

    pub async fn Add_to_group(
        &self,
        user_identifier: User_identifier_type,
        group_identifier: Group_identifier_type,
    ) -> Result_type<()> {
        if !self.Exists_group(group_identifier).await? {
            return Err(Error_type::Invalid_group_identifier);
        }
        let mut Inner = self.0.write().await;
        if !Inner
            .groups
            .get_mut(&group_identifier)
            .unwrap()
            .users
            .insert(user_identifier)
        {
            return Err(Error_type::Duplicate_group_identifier);
        }
        Ok(())
    }

    pub async fn Get_group_name(&self, Identifier: Group_identifier_type) -> Result_type<String> {
        Ok(self
            .0
            .read()
            .await
            .groups
            .get(&Identifier)
            .unwrap()
            .name
            .clone())
    }

    pub async fn Get_user_identifier(&self, Name: &str) -> Result_type<User_identifier_type> {
        Ok(*self
            .0
            .read()
            .await
            .users
            .iter()
            .find(|(_, User)| User.name == Name)
            .ok_or(Error_type::Invalid_user_identifier)?
            .0)
    }

    pub async fn Get_group_users(
        &self,
        identifier: Group_identifier_type,
    ) -> Result_type<Vec<User_identifier_type>> {
        Ok(self
            .0
            .read()
            .await
            .groups
            .get(&identifier)
            .ok_or(Error_type::Invalid_group_identifier)?
            .users
            .clone()
            .into_iter()
            .collect())
    }

    pub async fn Get_user_name(&self, Identifier: User_identifier_type) -> Result_type<String> {
        Ok(self
            .0
            .read()
            .await
            .users
            .get(&Identifier)
            .ok_or(Error_type::Invalid_user_identifier)?
            .name
            .clone())
    }

    pub async fn Get_user_primary_group(
        &self,
        identifier: User_identifier_type,
    ) -> Result_type<Group_identifier_type> {
        Ok(self
            .0
            .read()
            .await
            .users
            .get(&identifier)
            .ok_or(Error_type::Invalid_user_identifier)?
            .primary_group)
    }

    pub fn Check_credentials(&self, _User_name: &str, _Password: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    use Task::Test;

    #[Test]
    async fn Create_user() {
        let Manager = Manager_type::new();
        let User_name = "Alice";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::ROOT)
            .await
            .unwrap();
        assert!(Manager.Exists_user(Identifier).await.unwrap());
    }

    #[Test]
    async fn Create_user_duplicate() {
        let User_name_1 = "Alice";
        let User_name_2 = "Bob";

        let Identifier_1 = User_identifier_type::New(1000);
        let Identifier_2 = User_identifier_type::New(1001);

        // Same identifier
        let Manager = Manager_type::new();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::ROOT)
            .await
            .unwrap();

        let Result = Manager
            .Add_user(Identifier_1, User_name_2, Group_identifier_type::ROOT)
            .await;
        assert_eq!(Result, Err(Error_type::Duplicate_user_identifier));

        // Same name
        let Manager = Manager_type::new();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::ROOT)
            .await
            .unwrap();

        let Result = Manager
            .Add_user(Identifier_2, User_name_1, Group_identifier_type::ROOT)
            .await;
        assert_eq!(Result, Err(Error_type::Duplicate_user_name));

        // Same name and identifier
        let Manager = Manager_type::new();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::ROOT)
            .await
            .unwrap();

        Manager
            .Add_user(Identifier_1, User_name_1, Group_identifier_type::ROOT)
            .await
            .unwrap_err();
    }

    #[Test]
    async fn Create_group() {
        let Manager = Manager_type::new();
        let Group_name = "Developers";
        let Group_id = Group_identifier_type::New(1000);
        let Result = Manager.Add_group(Group_id, Group_name, &[]).await;
        assert!(Result.is_ok());
        assert!(Manager.Exists_group(Group_id).await.unwrap());
    }

    #[Test]
    async fn Create_group_duplicate() {
        let Group_name_1 = "Developers";
        let Group_name_2 = "Testers";

        let Group_id_1 = Group_identifier_type::New(1000);
        let Group_id_2 = Group_identifier_type::New(1001);

        // Same identifier
        let Manager = Manager_type::new();

        Manager
            .Add_group(Group_id_1, Group_name_1, &[])
            .await
            .unwrap();

        let Result = Manager.Add_group(Group_id_1, Group_name_2, &[]).await;
        assert_eq!(Result, Err(Error_type::Duplicate_group_identifier));

        // Same name
        let Manager = Manager_type::new();

        Manager
            .Add_group(Group_id_1, Group_name_1, &[])
            .await
            .unwrap();

        let Result = Manager.Add_group(Group_id_2, Group_name_1, &[]).await;
        assert_eq!(Result, Err(Error_type::Duplicate_group_name));

        // Same name and identifier
        let Manager = Manager_type::new();

        Manager
            .Add_group(Group_id_1, Group_name_1, &[])
            .await
            .unwrap();

        Manager
            .Add_group(Group_id_1, Group_name_1, &[])
            .await
            .unwrap_err();
    }

    #[Test]
    async fn Is_root() {
        let Root_id = User_identifier_type::ROOT;
        assert!(Manager_type::Is_root(Root_id));
    }

    #[Test]
    async fn Is_in_group() {
        let Manager = Manager_type::new();
        let User_name = "Bob";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::ROOT)
            .await
            .unwrap();
        let Group_name = "Admins";
        let Group_id = Group_identifier_type::New(1000);

        Manager.Add_group(Group_id, Group_name, &[]).await.unwrap();
        Manager.Add_to_group(Identifier, Group_id).await.unwrap();
        assert!(Manager.Is_in_group(Identifier, Group_id).await);
    }

    #[Test]
    async fn Get_user_groups() {
        let Manager = Manager_type::new();

        let User_name = "Charlie";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::ROOT)
            .await
            .unwrap();
        let Group_name1 = "TeamA";
        let Group_id1 = Group_identifier_type::New(1000);

        Manager
            .Add_group(Group_id1, Group_name1, &[])
            .await
            .unwrap();
        let Group_name2 = "TeamB";
        let Group_id2 = Group_identifier_type::New(1001);

        Manager
            .Add_group(Group_id2, Group_name2, &[])
            .await
            .unwrap();
        Manager.Add_to_group(Identifier, Group_id1).await.unwrap();
        Manager.Add_to_group(Identifier, Group_id2).await.unwrap();
        let Groups = Manager.Get_user_groups(Identifier).await.unwrap();

        assert_eq!(Groups.len(), 3);
        assert!(
            Groups.contains(&Group_id1)
                && Groups.contains(&Group_id2)
                && Groups.contains(&Group_identifier_type::ROOT)
        );
    }

    #[Test]
    async fn Get_group_name() {
        let Manager = Manager_type::new();
        let Group_name = "QA";
        let Group_id = Group_identifier_type::New(1000);
        Manager.Add_group(Group_id, Group_name, &[]).await.unwrap();
        let Retrieved_name = Manager.Get_group_name(Group_id).await.unwrap();
        assert_eq!(Group_name, Retrieved_name);
    }

    #[Test]
    async fn Get_group_users() {
        let Manager = Manager_type::new();
        let User_name = "Dave";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::ROOT)
            .await
            .unwrap();
        let Group_name = "Engineers";
        let Group_id = Group_identifier_type::New(1000);
        Manager.Add_group(Group_id, Group_name, &[]).await.unwrap();
        Manager.Add_to_group(Identifier, Group_id).await.unwrap();
        let Users = Manager.Get_group_users(Group_id).await.unwrap();
        assert_eq!(Users.len(), 1);
        assert!(Users.contains(&Identifier));
    }

    #[Test]
    async fn Get_user_name() {
        let Manager = Manager_type::new();
        let User_name = "Eve";
        let Identifier = User_identifier_type::New(1000);
        Manager
            .Add_user(Identifier, User_name, Group_identifier_type::ROOT)
            .await
            .unwrap();
        let Retrieved_name = Manager.Get_user_name(Identifier).await.unwrap();
        assert_eq!(User_name, Retrieved_name);
    }

    #[Test]
    async fn Check_credentials() {
        let Manager = Manager_type::new();
        let User_name = "Frank";
        let Password = "password123";
        assert!(Manager.Check_credentials(User_name, Password));
    }
}
