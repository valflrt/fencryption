use clap::Args as ClapArgs;

#[derive(ClapArgs)]
pub struct Args {}

pub fn action() {
    println!("Pong !");
}
