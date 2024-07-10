#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use env_logger::{Builder, Env};
use std::{collections::VecDeque, io::Write, process};

mod Target;

mod Command;

fn init_logger() {
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

fn Print_usage() {
    log::error!("Usage: xila <command> [target] [options]");
}

fn main() -> Result<(), ()> {
    init_logger();

    let mut Arguments: VecDeque<String> = std::env::args().collect();

    // Remove the first argument which is the program path
    Arguments.pop_front();

    if Arguments.len() < 1 {
        Print_usage();
        return Err(());
    }

    // Get the command
    let Command = Command::Command_type::try_from(Arguments.pop_front().unwrap().as_str())
        .map_err(|Error| log::error!("{}", Error))?;

    // Check if the target is needed
    let Target = if Command.Is_target_needed() {
        Some(
            Target::Target_type::try_from(
                Arguments
                    .pop_front()
                    .unwrap_or("native".to_string())
                    .as_str(),
            )
            .map_err(|Error| log::error!("{}", Error))?,
        )
    } else {
        None
    };

    // Create a new process::Command
    let mut Shell_command = process::Command::new("cargo");

    // Inherit the standard input, output and error
    Shell_command
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit());

    // Add the toolchain like +esp
    if let Some(Target) = Target {
        Shell_command.arg(Target.Get_toolchain());
    }

    // Add the cargo command like build, clean, run, test, fmt, doc
    Shell_command.arg(Command.Get_cargo_command());

    if let Some(Target) = Target {
        log::trace!("Target arguments : {:?}", Target.Get_target_arguments());

        // Add the target arguments like --target, -Z build-std=std,panic_abort
        Shell_command.args(Target.Get_target_arguments());

        log::trace!(
            "Environment variables : {:?}",
            Target.Get_environment_variables()
        );

        // Add the environment variables like MCU=esp32
        Shell_command.envs(Target.Get_environment_variables());
    }

    Shell_command.args(Arguments); // Add the remaining arguments

    log::trace!("Full command : {:?}", Shell_command);

    let mut Child = Shell_command
        .spawn()
        .map_err(|Error| log::error!("Error while executing cargo : {}", Error))?;

    Child.wait().map_err(|Error| log::error!("{}", Error))?;

    log::info!(
        "`{:?}` command executed successfully for target {:?}.",
        Command,
        Target.unwrap_or_default()
    );

    Ok(())
}
