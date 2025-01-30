use std::num::NonZeroUsize;

use Executable::{Execute, Standard_type};
use File_system::Path_type;

use crate::{
    Error_type, Parser::Parse, Resolver::Resolve, Result_type, Shell_type, Tokenizer::Tokenize,
};

pub fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::New(Standard).Main(Arguments)
}

impl Shell_type {
    pub fn New(Standard: Standard_type) -> Self {
        Self {
            Standard,
            Current_directory: Path_type::Root.to_owned(),
            Running: true,
            User: "".to_string(),
            Host: "".to_string(),
        }
    }

    fn Run(&mut self, Path: &Path_type, Arguments: &[&str]) -> Result_type<()> {
        let Standard = self.Standard.Duplicate().unwrap();

        let Input = Arguments.join(" ");

        let _ = Execute(Path, Input, Standard)
            .map_err(|_| Error_type::Failed_to_execute_command)?
            .Join()
            .map_err(|_| Error_type::Failed_to_join_task)?;

        Ok(())
    }

    fn Parse_input(&mut self, Input: &str, Paths: &[&Path_type]) -> Result_type<()> {
        let Tokens = Input.split_whitespace().collect::<Vec<&str>>();

        let Tokens = Tokenize(&Tokens);
        let Commands = Parse(Tokens)?;

        for Command in Commands {
            match Command.Get_command() {
                "exit" => self.Exit(Command.Get_arguments()),
                "cd" => self.Change_directory(Command.Get_arguments()),
                "echo" => self.Echo(Command.Get_arguments()),
                "ls" => self.List(Command.Get_arguments()),
                "clear" => self.Clear(Command.Get_arguments()),
                "cat" => self.Concatenate(Command.Get_arguments()),
                "stat" => self.Statistics(Command.Get_arguments()),
                "mkdir" => self.Create_directory(Command.Get_arguments()),
                "export" => self.Set_environment_variable(Command.Get_arguments()),
                "unset" => self.Remove_environment_variable(Command.Get_arguments()),
                "rm" => self.Remove(Command.Get_arguments()),
                _ => {
                    // - Set the current directory for the following commands.
                    if let Err(Error) = Task::Get_instance().Set_environment_variable(
                        self.Standard.Get_task(),
                        "Current_directory",
                        self.Current_directory.As_str(),
                    ) {
                        self.Standard.Print_error_line(&format!(
                            "Failed to set current directory: {}",
                            Error
                        ));
                    }

                    let Path = Path_type::From_str(Command.Get_command());

                    if Path.Is_valid() {
                        if Path.Is_absolute() {
                            self.Run(Path, Command.Get_arguments())?;
                        } else {
                            match self.Current_directory.clone().Join(Path) {
                                Some(Path) => self.Run(&Path, Command.Get_arguments())?,
                                None => self.Standard.Print_error_line("Invalid command"),
                            }
                        }
                    } else {
                        let Path = Resolve(Command.Get_command(), Paths)?;

                        self.Run(&Path, Command.Get_arguments())?;
                    }
                }
            }
        }

        Ok(())
    }

    fn Main_interactive(&mut self, Paths: &[&Path_type]) -> Result<(), Error_type> {
        let mut Input = String::new();

        while self.Running {
            self.Standard.Print(&format!(
                "{}@{}:{}$ ",
                self.User, self.Host, self.Current_directory
            ));

            self.Standard.Out_flush();

            Input.clear();

            self.Standard.Read_line(&mut Input);

            if Input.is_empty() {
                continue;
            }

            let Result = self.Parse_input(&Input, Paths);

            if let Err(Error) = Result {
                self.Standard.Print_error_line(&Error.to_string());
            }
        }

        Ok(())
    }

    pub fn Main(&mut self, Arguments: String) -> Result<(), NonZeroUsize> {
        let User =
            match Task::Get_instance().Get_environment_variable(self.Standard.Get_task(), "User") {
                Ok(User) => User.Get_value().to_string(),
                Err(_) => loop {
                    match self.Authenticate() {
                        Ok(User) => break User,
                        Err(Error) => self.Standard.Print_error_line(&Error.to_string()),
                    }
                },
            };

        self.User = User;

        let Paths = Task::Get_instance()
            .Get_environment_variable(self.Standard.Get_task(), "Paths")
            .map_err(|_| Error_type::Failed_to_get_path)?;

        let Paths = Paths
            .Get_value()
            .split(':')
            .map(Path_type::From_str)
            .collect::<Vec<&Path_type>>();

        let Host = Task::Get_instance()
            .Get_environment_variable(self.Standard.Get_task(), "Host")
            .map_err(|_| Error_type::Failed_to_get_path)?;
        self.Host = Host.Get_value().to_string();

        if Arguments.is_empty() {
            self.Main_interactive(&Paths)?;
        } else {
            self.Parse_input(&Arguments, &Paths)?;
        }

        Ok(())
    }
}
