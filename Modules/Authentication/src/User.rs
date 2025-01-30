use miniserde::{Deserialize, Serialize};
use Users::{
    Group_identifier_inner_type, Group_identifier_type, User_identifier_inner_type,
    User_identifier_type,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User_type {
    Identifier: User_identifier_inner_type,
    Name: String,
    Primary_group: Group_identifier_inner_type,
    Hash: String,
    Salt: String,
}

impl User_type {
    pub fn New(
        Identifier: User_identifier_inner_type,
        Name: String,
        Primary_group: Group_identifier_inner_type,
        Hash: String,
        Salt: String,
    ) -> Self {
        Self {
            Identifier,
            Name,
            Primary_group,
            Hash,
            Salt,
        }
    }

    pub fn Get_identifier(&self) -> User_identifier_type {
        User_identifier_type::New(self.Identifier)
    }

    pub fn Get_primary_group(&self) -> Group_identifier_type {
        Group_identifier_type::New(self.Primary_group)
    }

    pub fn Get_name(&self) -> &str {
        &self.Name
    }

    pub fn Get_hash(&self) -> &str {
        &self.Hash
    }

    pub fn Get_salt(&self) -> &str {
        &self.Salt
    }

    pub fn Set_hash(&mut self, Hash: String) {
        self.Hash = Hash;
    }

    pub fn Set_salt(&mut self, Salt: String) {
        self.Salt = Salt;
    }

    pub fn Set_primary_group(&mut self, Primary_group: Group_identifier_inner_type) {
        self.Primary_group = Primary_group;
    }

    pub fn Set_name(&mut self, Name: String) {
        self.Name = Name;
    }
}
