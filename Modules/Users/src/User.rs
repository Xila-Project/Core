use super::*;

pub struct User_type<'a> {
    User_manager: &'a Manager_type,
    Identifier: User_identifier_type,
}

impl<'a> User_type<'a> {
    pub fn New(User_manager: &'a Manager_type, Identifier: User_identifier_type) -> Self {
        User_type {
            User_manager,
            Identifier,
        }
    }

    pub fn Get_name(&self) -> String {
        self.User_manager.Get_user_name(self.Identifier).unwrap()
    }
}
