use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "tab")]
#[command(about = "Many-to-one payment", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true)]
    url: Option<String>,
    keypair: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Version,
    CreateJob {
        #[arg(short, long)]
        payers: String,
        #[arg(short, long)]
        payees: String,
        #[arg(short, long)]
        amount: f64,
        #[arg(short, long)]
        deadline: Option<i64>,
    },
    Pay {
        #[arg(short, long)]
        job_id: String,
    },
    Status {
        #[arg(short, long)]
        job_id: String,
    },
    Distribute {
        #[arg(short, long)]
        job_id: String,
    },
    Qr {
        #[arg(short, long)]
        job_id: String,
        #[arg(short, long)]
        output: Option<String>,
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            println!("cli version 0.0.0");
        },
        Commands::CreateJob { payers, payees, amount, deadline } => {
            commands::create_job(payers, payees, amount, deadline)?;
        },
        Commands::Pay { job_id } => {
            commands::pay(job_id)?;
        },
        Commands::Status { job_id } => {
            commands::status(job_id)?;
        },
        Commands::Distribute { job_id } => {
            commands::distribute(job_id)?;
        },
        Commands::Qr { job_id, output } => {
            commands::qr(job_id, output)?;
        },
    }

    Ok(())
}