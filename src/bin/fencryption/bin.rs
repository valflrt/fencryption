mod cli;
mod commands;
mod error;
mod log;
mod logic;
mod result;

// #[cfg(test)]
// mod tests;

fn main() {
    cli::parse();
}
