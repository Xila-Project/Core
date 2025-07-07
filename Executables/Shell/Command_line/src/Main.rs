use core::num::NonZeroUsize;

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};
use Executable::{Execute, Standard_type};
use File_system::Path_type;

use crate::{
    Error_type, Parser::Parse, Resolver::Resolve, Result_type, Shell_type, Tokenizer::Tokenize,
};

pub async fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::new(Standard).Main(Arguments).await
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

    async fn Run(&mut self, Path: &Path_type, Arguments: &[&str]) -> Result_type<()> {
        let standard = self.standard.Duplicate().await.unwrap();

        let Input = Arguments.join(" ");

        let _ = Execute(Path, Input, standard)
            .await
            .map_err(|_| Error_type::Failed_to_execute_command)?
            .Join()
            .await;

        Ok(())
    }

    async fn Parse_input(&mut self, Input: &str, Paths: &[&Path_type]) -> Result_type<()> {
        let tokens = Input.split_whitespace().collect::<Vec<&str>>();

        let Tokens = Tokenize(&tokens);
        let commands = Parse(Tokens)?;

        for Command in commands {
            match Command.get_command() {
                "exit" => self.exit(Command.Get_arguments()).await,
                "cd" => self.change_directory(Command.Get_arguments()).await,
                "echo" => self.echo(Command.Get_arguments()).await,
                "ls" => self.list(Command.Get_arguments()).await,
                "clear" => self.clear(Command.Get_arguments()).await,
                "cat" => self.Concatenate(Command.Get_arguments()).await,
                "stat" => self.statistics(Command.Get_arguments()).await,
                "mkdir" => self.create_directory(Command.Get_arguments()).await,
                "export" => self.set_environment_variable(Command.Get_arguments()).await,
                "unset" => {
                    self.Remove_environment_variable(Command.Get_arguments())
                        .await
                }
                "rm" => self.Remove(Command.Get_arguments()).await,
                _ => {
                    // - Set the current directory for the following commands.
                    if let Err(Error) = Task::Get_instance()
                        .Set_environment_variable(
                            self.standard.Get_task(),
                            "Current_directory",
                            self.current_directory.As_str(),
                        )
                        .await
                    {
                        self.standard
                            .Print_error_line(&format!("Failed to set current directory: {Error}"))
                            .await;
                    }

                    let Path = Path_type::From_str(Command.get_command());

                    if Path.Is_valid() {
                        if Path.Is_absolute() {
                            self.Run(Path, Command.Get_arguments()).await?;
                        } else {
                            match self.current_directory.clone().Join(Path) {
                                Some(path) => self.Run(&path, Command.Get_arguments()).await?,
                                None => self.standard.Print_error_line("Invalid command").await,
                            }
                        }
                    } else {
                        let path = Resolve(Command.get_command(), Paths).await?;

                        self.Run(&path, Command.Get_arguments()).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn Main_interactive(&mut self, Paths: &[&Path_type]) -> Result<(), Error_type> {
        let mut input = String::new();

        while self.running {
            self.standard
                .Print(&format!(
                    "{}@{}:{}$ ",
                    self.user, self.host, self.current_directory
                ))
                .await;

            self.standard.Out_flush().await;

            input.clear();

            self.standard.Read_line(&mut input).await;

            if input.is_empty() {
                continue;
            }

            let Result = self.Parse_input(&input, Paths).await;

            if let Err(Error) = Result {
                self.standard.Print_error_line(&Error.to_string()).await;
            }
        }

        Ok(())
    }

    pub async fn Main(&mut self, Arguments: String) -> Result<(), NonZeroUsize> {
        let user = match Task::Get_instance()
            .Get_environment_variable(self.standard.Get_task(), "User")
            .await
        {
            Ok(User) => User.Get_value().to_string(),
            Err(_) => loop {
                match self.authenticate().await {
                    Ok(user) => break user,
                    Err(error) => self.standard.Print_error_line(&error.to_string()).await,
                }
            },
        };

        self.user = user;

        let Paths = Task::Get_instance()
            .Get_environment_variable(self.standard.Get_task(), "Paths")
            .await
            .map_err(|_| Error_type::Failed_to_get_path)?;

        let Paths = Paths
            .Get_value()
            .split(':')
            .map(Path_type::From_str)
            .collect::<Vec<&Path_type>>();

        let Host = Task::Get_instance()
            .Get_environment_variable(self.standard.Get_task(), "Host")
            .await
            .map_err(|_| Error_type::Failed_to_get_path)?;
        self.host = Host.Get_value().to_string();

        if Arguments.is_empty() {
            self.Main_interactive(&Paths).await?;
        } else {
            self.Parse_input(&Arguments, &Paths).await?;
        }

        Ok(())
    }
}
