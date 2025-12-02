use file_system::{Attributes, Kind, Permissions};
use users::{GroupIdentifier, UserIdentifier};

pub fn get_attributes(index: usize, attributes: &mut Attributes) {
    if let Some(kind) = attributes.get_mutable_kind() {
        *kind = Kind::CharacterDevice;
    }
    if let Some(size) = attributes.get_mutable_size() {
        *size = 0;
    }
    if let Some(permissions) = attributes.get_mutable_permissions() {
        *permissions = Permissions::ALL_READ_WRITE;
    }
    if let Some(links) = attributes.get_mutable_links() {
        *links = 1;
    }
    if let Some(inode) = attributes.get_mutable_inode() {
        *inode = index as u64 + 1;
    }
    if let Some(user) = attributes.get_mutable_user() {
        *user = UserIdentifier::ROOT;
    }
    if let Some(group) = attributes.get_mutable_group() {
        *group = GroupIdentifier::ROOT;
    }
}
