mod cli;
mod commands;
mod error;
mod log;
mod logic;
mod metadata_structs;
mod result;

// #[cfg(test)]
// pub mod tests;

fn main() {
    cli::parse();
}
