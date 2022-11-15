use std::io::{self, stdin, stdout, Write};

use colored::Colorize;

enum LogKind {
    Info,
    Success,
    Error,
}

/// Adds a "start line pattern" at the start of every line in
/// the given text.
pub fn with_start_line<T, L>(text: T, line_start: L) -> String
where
    T: AsRef<str>,
    L: AsRef<str>,
{
    format!(
        "{} {}",
        line_start.as_ref(),
        text.as_ref()
            .replace("\n", &["\n", line_start.as_ref(), " "].concat())
    )
}

fn format_message<M>(message: M, kind: LogKind) -> String
where
    M: AsRef<str>,
{
    if supports_color::on(supports_color::Stream::Stdout).is_some() && cfg!(target_family = "unix")
    {
        with_start_line(
            message,
            match kind {
                LogKind::Info => " ".on_white(),
                LogKind::Success => " ".on_bright_green(),
                LogKind::Error => " ".on_bright_red(),
            }
            .to_string(),
        )
    } else {
        message.as_ref().to_string()
    }
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

pub fn prompt<M>(message: M) -> io::Result<String>
where
    M: AsRef<str>,
{
    print!("{}", format_info(message));
    stdout().flush()?;
    let mut out = String::new();
    stdin().read_line(&mut out)?;
    Ok(out)
}
