use crate::{Error, Result, Shell};
use alloc::string::ToString;
use core::fmt::Write;
use executable_macros::GetArgs;
use xila::{
    file_system::{AccessFlags, Kind, Path},
    log, users,
    virtual_file_system::{self, Directory, File},
};

#[derive(GetArgs)]
struct ListArguments<'a> {
    #[arg(positional, default = "")]
    path: &'a str,
    #[arg(flag, short = 'c', long = "characters")]
    characters: bool,
    #[arg(flag, short = 'w', long = "words")]
    words: bool,
    #[arg(flag, short = 'l', long = "lines")]
    lines: bool,
    #[arg(flag, short = 'L', long = "longest-line")]
    longest_line: bool,
}

impl Shell {
    pub async fn word_count<'a, I>(
        &mut self,
        options: &mut getargs::Options<&'a str, I>,
    ) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let ListArguments {
            path,
            characters,
            words,
            lines,
            longest_line,
        } = ListArguments::parse(options)?;
        let path: &Path = if path.is_empty() {
            self.current_directory.as_ref()
        } else {
            Path::from_str(path)
        };

        let virtual_file_system = virtual_file_system::get_instance();

        let mut file = File::open(
            virtual_file_system,
            self.task,
            &path,
            AccessFlags::Read.into(),
        )
        .await
        .map_err(Error::FailedToOpenFile)?;

        let mut character_count = 0;
        let mut word_count = 0;
        let mut line_count = 0;
        let mut longest_line_length = 0;
        let mut current_line_length = 0;
        let mut in_word = false;

        let mut buffer = [0u8; 256];
        loop {
            let bytes_read = file
                .read(&mut buffer)
                .await
                .map_err(Error::FailedToReadFile)?;

            if bytes_read == 0 {
                break;
            }

            for &byte in &buffer[..bytes_read] {
                character_count += 1;

                if byte == b'\n' {
                    line_count += 1;
                    longest_line_length = longest_line_length.max(current_line_length);
                    current_line_length = 0;
                } else {
                    current_line_length += 1;
                }

                if byte.is_ascii_whitespace() {
                    if in_word {
                        word_count += 1;
                        in_word = false;
                    }
                } else {
                    in_word = true;
                }
            }
        }
        if in_word {
            word_count += 1;
        }

        if characters {
            write!(self.standard.out(), "{} ", character_count)?;
        }
        if words {
            write!(self.standard.out(), "{} ", word_count)?;
        }
        if lines {
            write!(self.standard.out(), "{} ", line_count)?;
        }
        if longest_line {
            write!(self.standard.out(), "{}", longest_line_length)?;
        }
        writeln!(self.standard.out())?;

        Ok(())
    }
}
