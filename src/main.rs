mod actions;
mod cli;
mod commands;
mod log;

#[cfg(test)]
pub mod bin_tests;

fn main() {
    cli::parse();
}
