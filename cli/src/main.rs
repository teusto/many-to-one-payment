use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tab")]
#[command(about = "Many-to-one payment", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Version,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            println!("cli version 0.0.0");
        }
    }
}