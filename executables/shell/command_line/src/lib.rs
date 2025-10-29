#![no_std]

extern crate alloc;

xila::internationalization::include_translations!();

use core::num::NonZeroUsize;

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};
use xila::executable::Standard;
use xila::file_system::Path;
use xila::task;

use crate::{Error, Result, parser::parse, tokenizer::tokenize};

mod commands;
mod error;

mod parser;
mod resolver;
mod tokenizer;

use error::*;
use xila::{executable, file_system::PathOwned};

pub struct Shell {
    standard: Standard,
    current_directory: PathOwned,
    running: bool,
    user: String,
    host: String,
}

pub struct ShellExecutable;

executable::implement_executable_device!(
    structure: ShellExecutable,
    mount_path: "/binaries/command_line_shell",
    main_function: main,
);

pub async fn main(
    standard: Standard,
    arguments: Vec<String>,
) -> core::result::Result<(), NonZeroUsize> {
    Shell::new(standard).main(arguments).await
}

impl Shell {
    pub fn new(standard: Standard) -> Self {
        Self {
            standard,
            current_directory: Path::ROOT.to_owned(),
            running: true,
            user: "".to_string(),
            host: "".to_string(),
        }
    }

    async fn parse_input<'a, I>(&mut self, input: I, paths: &[&Path]) -> Result<()>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let tokens = tokenize(input);
        let commands = parse(tokens)?;

        for command in commands {
            match command.command {
                "exit" => self.exit(&command.arguments).await,
                "cd" => self.change_directory(&command.arguments).await,
                "echo" => self.echo(&command.arguments).await,
                "ls" => self.list(&command.arguments).await,
                "clear" => self.clear(&command.arguments).await,
                "cat" => self.concatenate(&command.arguments).await,
                "stat" => self.statistics(&command.arguments).await,
                "mkdir" => self.create_directory(&command.arguments).await,
                "export" => self.set_environment_variable(&command.arguments).await,
                "unset" => self.remove_environment_variable(&command.arguments).await,
                "rm" => self.remove(&command.arguments).await,
                _ => self.execute(command, paths).await?,
            }
        }

        Ok(())
    }

    async fn main_interactive(&mut self, paths: &[&Path]) -> Result<()> {
        let mut input_string = String::new();

        while self.running {
            self.standard
                .print(&format!(
                    "{}@{}:{}$ ",
                    self.user, self.host, self.current_directory
                ))
                .await;

            self.standard.out_flush().await;

            input_string.clear();

            self.standard.read_line(&mut input_string).await;

            if input_string.is_empty() {
                continue;
            }

            let input = input_string.split(" ");

            let result = self.parse_input(input, paths).await;

            if let Err(error) = result {
                self.standard.print_error_line(&error.to_string()).await;
            }
        }

        Ok(())
    }

    pub async fn main(&mut self, arguments: Vec<String>) -> core::result::Result<(), NonZeroUsize> {
        let user = match task::get_instance()
            .get_environment_variable(self.standard.get_task(), "User")
            .await
        {
            Ok(user) => user.get_value().to_string(),
            Err(_) => loop {
                match self.authenticate().await {
                    Ok(user) => break user,
                    Err(error) => self.standard.print_error_line(&error.to_string()).await,
                }
            },
        };

        self.user = user;

        let paths = task::get_instance()
            .get_environment_variable(self.standard.get_task(), "Paths")
            .await
            .map_err(|_| Error::FailedToGetPath)?;

        let paths = paths
            .get_value()
            .split(':')
            .map(Path::from_str)
            .collect::<Vec<&Path>>();

        let host = task::get_instance()
            .get_environment_variable(self.standard.get_task(), "Host")
            .await
            .map_err(|_| Error::FailedToGetPath)?;
        self.host = host.get_value().to_string();

        if arguments.is_empty() {
            self.main_interactive(&paths).await?;
        } else {
            let arguments = arguments.iter().map(|s| s.as_str());
            self.parse_input(arguments, &paths).await?;
        }

        Ok(())
    }
}
