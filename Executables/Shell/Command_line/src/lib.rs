#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use executable::Standard_type;

use core::num::NonZeroUsize;

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};
use executable::execute;
use file_system::Path_type;

use crate::{parser::parse, resolver::resolve, tokenizer::tokenize, Error_type, Result_type};

mod commands;
mod error;

mod parser;
mod resolver;
mod tokenizer;

use error::*;
use file_system::Path_owned_type;

pub struct Shell_type {
    standard: Standard_type,
    current_directory: Path_owned_type,
    running: bool,
    user: String,
    host: String,
}

pub struct Shell_executable_type;

executable::Implement_executable_device!(
    Structure: Shell_executable_type,
    Mount_path: "/Binaries/Command_line_shell",
    Main_function: main,
);

pub async fn main(standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::new(standard).main(Arguments).await
}

impl Shell_type {
    pub fn new(standard: Standard_type) -> Self {
        Self {
            standard,
            current_directory: Path_type::ROOT.to_owned(),
            running: true,
            user: "".to_string(),
            host: "".to_string(),
        }
    }

    async fn run(&mut self, path: &Path_type, arguments: &[&str]) -> Result_type<()> {
        let standard = self.standard.duplicate().await.unwrap();

        let input = arguments.join(" ");

        let _ = execute(path, input, standard)
            .await
            .map_err(|_| Error_type::Failed_to_execute_command)?
            .Join()
            .await;

        Ok(())
    }

    async fn parse_input(&mut self, input: &str, paths: &[&Path_type]) -> Result_type<()> {
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
                        .Set_environment_variable(
                            self.standard.get_task(),
                            "Current_directory",
                            self.current_directory.As_str(),
                        )
                        .await
                    {
                        self.standard
                            .print_error_line(&format!("Failed to set current directory: {error}"))
                            .await;
                    }

                    let path = Path_type::From_str(command.get_command());

                    if path.is_valid() {
                        if path.is_absolute() {
                            self.run(path, command.get_arguments()).await?;
                        } else {
                            match self.current_directory.clone().Join(path) {
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

    async fn main_interactive(&mut self, Paths: &[&Path_type]) -> Result<(), Error_type> {
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

            let Result = self.parse_input(&input, Paths).await;

            if let Err(error) = Result {
                self.standard.print_error_line(&error.to_string()).await;
            }
        }

        Ok(())
    }

    pub async fn main(&mut self, arguments: String) -> Result<(), NonZeroUsize> {
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
            .map_err(|_| Error_type::Failed_to_get_path)?;

        let paths = paths
            .get_value()
            .split(':')
            .map(Path_type::From_str)
            .collect::<Vec<&Path_type>>();

        let host = task::get_instance()
            .get_environment_variable(self.standard.get_task(), "Host")
            .await
            .map_err(|_| Error_type::Failed_to_get_path)?;
        self.host = host.get_value().to_string();

        if arguments.is_empty() {
            self.main_interactive(&paths).await?;
        } else {
            self.parse_input(&arguments, &paths).await?;
        }

        Ok(())
    }
}
