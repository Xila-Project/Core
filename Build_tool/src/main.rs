



use env_logger::{Builder, Env};
use std::{collections::VecDeque, io::Write, process};
use command::Command_type;
use target::Target_type;

mod target;

mod command;

fn Initialize_logger() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let Colored_level = match record.level() {
                log::Level::Error => "❌",
                log::Level::Warn => "⚠️",
                log::Level::Info => "💡",
                log::Level::Debug => "🐛",
                log::Level::Trace => "🔍",
            };

            writeln!(buf, "{} - {}", Colored_level, record.args())
        })
        .init();
}

fn get_usage() -> &'static str {
    "Usage: xila [command] [target] [args]
    Commands:
        build       Compile the current project.
        clean       Remove the target directory.
        run         Build and execute the current project.
        test        Run the tests.
        format      Format the code.
        doc         Generate the documentation.
    Targets:
        esp32
        esp32s3
        linux
        windows
        native
    Arguments:
        Any arguments after the target are passed to corresponding cargo command.
    Example:
        xila build esp32 --release
        xila run esp32
        xila test
        xila format
        xila doc
    "
}

fn Parse_arguments() -> Result<(Command_type, Option<Target_type>, Vec<String>), String> {
    let mut Arguments: VecDeque<String> = std::env::args().collect();

    // Remove the first argument which is the program path
    Arguments.pop_front();

    let Command = Arguments
        .pop_front()
        .ok_or_else(|| "No command provided.".to_string())?;

    // Get the command
    let Command = Command::Command_type::try_from(Command.as_str())?;

    let Target = if Command.is_target_needed() && !Arguments.is_empty() {
        let Target: &String = Arguments.front().unwrap();

        let Target = Target::Target_type::try_from(Target.as_str())?;

        Arguments.pop_front();
        Some(Target)
    } else {
        None
    };

    Ok((Command, Target, Arguments.into_iter().collect()))
}

fn get_cargo_arguments(
    Command: Command_type,
    Target: Option<Target_type>,
    Arguments: &mut Vec<String>,
) -> Vec<String> {
    let mut Cargo_arguments = Vec::new();

    // Add the toolchain like +esp
    if let Some(Target) = Target {
        Cargo_arguments.push(Target.get_toolchain());
        log::trace!("Toolchain : {}", Target.get_toolchain());
    }

    // Add the cargo command like build, clean, run, test, fmt, doc
    Cargo_arguments.push(Command.get_cargo_command().expect("Unknown command"));

    // Add the target arguments like --target, -Z build-std=std,panic_abort
    if let Some(Target) = Target {
        Cargo_arguments.append(&mut Target.get_arguments());
        log::trace!("Target arguments : {:?}", Target.get_arguments());
    }

    // Add the remaining additional arguments
    Cargo_arguments.append(Arguments);
    log::trace!("Additional arguments : {:?}", Arguments);

    Cargo_arguments
}

fn main() -> Result<(), ()> {
    Initialize_logger();

    let (Command, Target, mut Arguments) = match Parse_arguments() {
        Ok(Arguments) => Arguments,
        Err(error) => {
            log::error!("{}\n{}", Error, get_usage());
            return Err(());
        }
    };

    if Command == Command_type::Help {
        println!("{}", get_usage());
        return Ok(());
    }

    // Create a new process::Command
    let mut Shell_command = process::Command::new("cargo");

    // Inherit the standard input, output and error
    Shell_command
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .args(Get_cargo_arguments(Command, Target, &mut Arguments));

    if let Some(Target) = Target {
        log::trace!(
            "Environment variables : {:?}",
            Target.get_environment_variables()
        );

        // Add the environment variables like MCU=esp32
        Shell_command.envs(Target.get_environment_variables());
    }

    log::trace!("Full command : {:?}", Shell_command);

    let mut Child = match Shell_command.spawn() {
        Ok(Child) => Child,
        Err(error) => {
            log::error!("Failed to spawn the cargo command : {}", Error);
            return Err(());
        }
    };

    let Status = match Child.wait() {
        Ok(Status) => Status,
        Err(error) => {
            log::error!("Failed to wait for the cargo command : {}", Error);
            return Err(());
        }
    };

    if !Status.success() {
        log::error!(
            "Failed to execute `{:?}` command for target {:?}.",
            Command,
            Target.unwrap_or_default()
        );
        return Err(());
    }

    log::info!(
        "`{:?}` command executed successfully for target {:?}.",
        Command,
        Target.unwrap_or_default()
    );

    Ok(())
}
