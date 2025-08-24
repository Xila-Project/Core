#![no_std]

extern crate alloc;

use executable::Standard;

use core::num::NonZeroUsize;

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};
use executable::execute;
use file_system::Path;

use crate::{Error, Result, parser::parse, resolver::resolve, tokenizer::tokenize};

mod commands;
mod error;

mod parser;
mod resolver;
mod tokenizer;

use error::*;
use file_system::PathOwned;

pub struct Shell {
    standard: Standard,
    current_directory: PathOwned,
    running: bool,
    user: String,
    host: String,
}

pub struct ShellExecutable;

executable::implement_executable_device!(
    Structure: ShellExecutable,
    Mount_path: "/binaries/command_line_shell",
    Main_function: main,
);

pub async fn main(standard: Standard, arguments: String) -> core::result::Result<(), NonZeroUsize> {
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

    async fn run(&mut self, path: &Path, arguments: &[&str]) -> Result<()> {
        let standard = self.standard.duplicate().await.unwrap();

        let input = arguments.join(" ");

        let _ = execute(path, input, standard)
            .await
            .map_err(|_| Error::FailedToExecuteCommand)?
            .join()
            .await;

        Ok(())
    }

    async fn parse_input(&mut self, input: &str, paths: &[&Path]) -> Result<()> {
        let tokens = input.split_whitespace().collect::<Vec<&str>>();

        let tokens = tokenize(&tokens);
        let commands = parse(tokens)?;

        for command in commands {
            match command.get_command() {
                "exit" => self.exit(command.get_arguments()).await,
                "cd" => self.change_directory(command.get_arguments()).await,
                "echo" => self.echo(command.get_arguments()).await,
                "ls" => self.list(command.get_arguments()).await,
                "clear" => self.clear(command.get_arguments()).await,
                "cat" => self.concatenate(command.get_arguments()).await,
                "stat" => self.statistics(command.get_arguments()).await,
                "mkdir" => self.create_directory(command.get_arguments()).await,
                "export" => self.set_environment_variable(command.get_arguments()).await,
                "unset" => {
                    self.remove_environment_variable(command.get_arguments())
                        .await
                }
                "rm" => self.remove(command.get_arguments()).await,
                _ => {
                    // - Set the current directory for the following commands.
                    if let Err(error) = task::get_instance()
                        .set_environment_variable(
                            self.standard.get_task(),
                            "Current_directory",
                            self.current_directory.as_str(),
                        )
                        .await
                    {
                        self.standard
                            .print_error_line(&format!("Failed to set current directory: {error}"))
                            .await;
                    }

                    let path = Path::from_str(command.get_command());

                    if path.is_valid() {
                        if path.is_absolute() {
                            self.run(path, command.get_arguments()).await?;
                        } else {
                            match self.current_directory.clone().join(path) {
                                Some(path) => self.run(&path, command.get_arguments()).await?,
                                None => self.standard.print_error_line("Invalid command").await,
                            }
                        }
                    } else {
                        let path = resolve(command.get_command(), paths).await?;

                        self.run(&path, command.get_arguments()).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn main_interactive(&mut self, paths: &[&Path]) -> Result<()> {
        let mut input = String::new();

        while self.running {
            self.standard
                .print(&format!(
                    "{}@{}:{}$ ",
                    self.user, self.host, self.current_directory
                ))
                .await;

            self.standard.out_flush().await;

            input.clear();

            self.standard.read_line(&mut input).await;

            if input.is_empty() {
                continue;
            }

            let result = self.parse_input(&input, paths).await;

            if let Err(error) = result {
                self.standard.print_error_line(&error.to_string()).await;
            }
        }

        Ok(())
    }

    pub async fn main(&mut self, arguments: String) -> core::result::Result<(), NonZeroUsize> {
        log::information!("Starting command line shell...");
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
            self.parse_input(&arguments, &paths).await?;
        }

        Ok(())
    }
}
