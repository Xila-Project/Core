use Users::Group_identifier_type;

pub struct User_type<'a> {
    Name: String,
    Password: String,
    Salt: String,
    Groups: &'a [u16],
}
