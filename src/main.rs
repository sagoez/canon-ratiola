use clap::{Parser, Subcommand};
use payment::{
    domain::OrchestratorMode,
    service::{mock::generator, orchestrator::Orchestrator},
};

#[derive(Parser, Debug)]
#[command(name = "payment", version, about = "A payment processing CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the transactions CSV file to process
    #[arg(value_name = "FILE")]
    file: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate dummy test data to a file
    Generate {
        /// Output file path
        #[arg(short, long, default_value = "transactions.csv", value_name = "FILE")]
        output: String,

        /// Number of transactions to generate
        #[arg(short, long, default_value = "10", value_name = "COUNT")]
        count: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Generate { output, count }) => {
            generator(&output, count)?;
        }
        None => {
            let file = args
                .file
                .ok_or("Please provide a CSV file path or use 'test' command")?;

            let orchestrator = Orchestrator::new(OrchestratorMode::Csv { file_path: file }).await;
            let final_states = orchestrator.process().await?;
            Orchestrator::output_csv(&final_states)?;
        }
    }

    Ok(())
}
