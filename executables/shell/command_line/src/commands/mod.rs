mod authentication;
mod change_directory;
mod clear;
mod concatenate;
mod directory;
mod dns;
mod echo;
mod environment_variables;
mod execute;
mod exit;
mod ip;
mod list;
mod ping;
mod statistics;
mod web_request;

fn check_no_more_arguments<'a, I>(options: &mut getargs::Options<&'a str, I>) -> crate::Result<()>
where
    I: Iterator<Item = &'a str>,
{
    if options.next_positional().is_some() {
        xila::log::information!("More arguments ! : {:?}", options.next_arg());
        return Err(crate::Error::InvalidNumberOfArguments);
    }

    Ok(())
}

fn check_no_more_options<'a, I>(options: &mut getargs::Options<&'a str, I>) -> crate::Result<()>
where
    I: Iterator<Item = &'a str>,
{
    if Ok(None) != options.next_opt() {
        return Err(crate::Error::InvalidOption);
    }

    Ok(())
}
