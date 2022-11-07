use colored::Colorize;

enum LogKind {
    Info,
    Success,
    Error,
}

fn format_message<M>(message: M, kind: LogKind) -> String
where
    M: AsRef<str>,
{
    let message = message.as_ref();
    let line_start = match kind {
        LogKind::Info => " ".black().on_white(),
        LogKind::Success => " ".black().on_bright_green(),
        LogKind::Error => " ".black().on_bright_red(),
    };
    format!(
        "{} {}",
        line_start,
        message.replace("\n", &["\n", &line_start, " "].concat())
    )
}

pub fn format_info<M>(message: M) -> String
where
    M: AsRef<str>,
{
    format_message(message, LogKind::Info)
}

pub fn println_info<M>(message: M)
where
    M: AsRef<str>,
{
    println!("{}", format_info(message));
}

pub fn format_success<M>(message: M) -> String
where
    M: AsRef<str>,
{
    format_message(message, LogKind::Success)
}

pub fn println_success<M>(message: M)
where
    M: AsRef<str>,
{
    println!("{}", format_success(message))
}

pub fn format_error<M>(message: M) -> String
where
    M: AsRef<str>,
{
    format_message(message, LogKind::Error)
}

pub fn println_error<M>(message: M)
where
    M: AsRef<str>,
{
    println!("{}", format_error(message))
}
