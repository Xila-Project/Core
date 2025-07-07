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
    Shell_type::New(Standard).Main(Arguments).await
}

impl Shell_type {
    pub fn New(Standard: Standard_type) -> Self {
        Self {
            Standard,
            Current_directory: Path_type::ROOT.to_owned(),
            Running: true,
            User: "".to_string(),
            Host: "".to_string(),
        }
    }

    async fn Run(&mut self, Path: &Path_type, Arguments: &[&str]) -> Result_type<()> {
        let Standard = self.Standard.Duplicate().await.unwrap();

        let Input = Arguments.join(" ");

        let _ = Execute(Path, Input, Standard)
            .await
            .map_err(|_| Error_type::Failed_to_execute_command)?
            .Join()
            .await;

        Ok(())
    }

    async fn Parse_input(&mut self, Input: &str, Paths: &[&Path_type]) -> Result_type<()> {
        let Tokens = Input.split_whitespace().collect::<Vec<&str>>();

        let Tokens = Tokenize(&Tokens);
        let Commands = Parse(Tokens)?;

        for Command in Commands {
            match Command.Get_command() {
                "exit" => self.Exit(Command.Get_arguments()).await,
                "cd" => self.Change_directory(Command.Get_arguments()).await,
                "echo" => self.Echo(Command.Get_arguments()).await,
                "ls" => self.List(Command.Get_arguments()).await,
                "clear" => self.Clear(Command.Get_arguments()).await,
                "cat" => self.Concatenate(Command.Get_arguments()).await,
                "stat" => self.Statistics(Command.Get_arguments()).await,
                "mkdir" => self.Create_directory(Command.Get_arguments()).await,
                "export" => self.Set_environment_variable(Command.Get_arguments()).await,
                "unset" => {
                    self.Remove_environment_variable(Command.Get_arguments())
                        .await
                }
                "rm" => self.Remove(Command.Get_arguments()).await,
                _ => {
                    // - Set the current directory for the following commands.
                    if let Err(Error) = Task::Get_instance()
                        .Set_environment_variable(
                            self.Standard.Get_task(),
                            "Current_directory",
                            self.Current_directory.As_str(),
                        )
                        .await
                    {
                        self.Standard
                            .Print_error_line(&format!("Failed to set current directory: {Error}"))
                            .await;
                    }

                    let Path = Path_type::From_str(Command.Get_command());

                    if Path.Is_valid() {
                        if Path.Is_absolute() {
                            self.Run(Path, Command.Get_arguments()).await?;
                        } else {
                            match self.Current_directory.clone().Join(Path) {
                                Some(Path) => self.Run(&Path, Command.Get_arguments()).await?,
                                None => self.Standard.Print_error_line("Invalid command").await,
                            }
                        }
                    } else {
                        let Path = Resolve(Command.Get_command(), Paths).await?;

                        self.Run(&Path, Command.Get_arguments()).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn Main_interactive(&mut self, Paths: &[&Path_type]) -> Result<(), Error_type> {
        let mut Input = String::new();

        while self.Running {
            self.Standard
                .Print(&format!(
                    "{}@{}:{}$ ",
                    self.User, self.Host, self.Current_directory
                ))
                .await;

            self.Standard.Out_flush().await;

            Input.clear();

            self.Standard.Read_line(&mut Input).await;

            if Input.is_empty() {
                continue;
            }

            let Result = self.Parse_input(&Input, Paths).await;

            if let Err(Error) = Result {
                self.Standard.Print_error_line(&Error.to_string()).await;
            }
        }

        Ok(())
    }

    pub async fn Main(&mut self, Arguments: String) -> Result<(), NonZeroUsize> {
        let User = match Task::Get_instance()
            .Get_environment_variable(self.Standard.Get_task(), "User")
            .await
        {
            Ok(User) => User.Get_value().to_string(),
            Err(_) => loop {
                match self.Authenticate().await {
                    Ok(User) => break User,
                    Err(Error) => self.Standard.Print_error_line(&Error.to_string()).await,
                }
            },
        };

        self.User = User;

        let Paths = Task::Get_instance()
            .Get_environment_variable(self.Standard.Get_task(), "Paths")
            .await
            .map_err(|_| Error_type::Failed_to_get_path)?;

        let Paths = Paths
            .Get_value()
            .split(':')
            .map(Path_type::From_str)
            .collect::<Vec<&Path_type>>();

        let Host = Task::Get_instance()
            .Get_environment_variable(self.Standard.Get_task(), "Host")
            .await
            .map_err(|_| Error_type::Failed_to_get_path)?;
        self.Host = Host.Get_value().to_string();

        if Arguments.is_empty() {
            self.Main_interactive(&Paths).await?;
        } else {
            self.Parse_input(&Arguments, &Paths).await?;
        }

        Ok(())
    }
}
