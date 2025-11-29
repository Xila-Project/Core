use crate::{Error, Result, Shell, commands::check_no_more_arguments};
use alloc::{borrow::ToOwned, format};
use core::fmt::Write;
use getargs::Options;
use xila::{
    file_system::Path,
    internationalization::translate,
    shared::{BYTES_SUFFIX, Unit},
    users, virtual_file_system,
};

impl Shell {
    pub async fn statistics<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let path = options
            .next_positional()
            .ok_or(Error::MissingPositionalArgument("path"))?;

        check_no_more_arguments(options)?;
        let path = Path::from_str(path);

        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            self.current_directory
                .clone()
                .join(path)
                .ok_or(Error::FailedToJoinPath)?
        };

        let statistics = virtual_file_system::get_instance()
            .get_statistics(&path)
            .await
            .map_err(Error::FailedToGetMetadata)?;

        let user = match users::get_instance().get_user_name(statistics.user).await {
            Ok(user) => user,
            Err(_) => {
                format!("{}", statistics.user.as_u16())
            }
        };

        let group = match users::get_instance().get_group_name(statistics.group).await {
            Ok(group) => group,
            Err(_) => {
                format!("{}", statistics.group.as_u16())
            }
        };

        let size = Unit::new(statistics.size as f32, BYTES_SUFFIX.name);

        let _ = writeln!(
            self.standard.out(),
            r#"{}: {} - {} : {}
{}: {} - {}: {}
{}: {} - {}: {} - {}: {}
{}: {}
{}: {}
{}: {}
{}: {}

"#,
            translate!("Kind"),
            statistics.kind,
            translate!("Inode"),
            statistics.inode,
            translate!("Links"),
            statistics.links,
            translate!("Size"),
            size,
            translate!("User"),
            user,
            translate!("Group"),
            group,
            translate!("Permissions"),
            statistics.permissions,
            translate!("Accessed"),
            statistics.access,
            translate!("Modified"),
            statistics.modification,
            translate!("Created"),
            statistics.creation,
            translate!("Status"),
            statistics.status,
        );

        Ok(())
    }
}
