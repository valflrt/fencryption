mod cli;
mod commands;

#[quit::main]
fn main() {
    cli::parse();
}
