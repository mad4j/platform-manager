use clap::{Parser, Subcommand};
use my_app_transport::{ActionRequest, ActionServiceClient};
use tonic::transport::Channel;
use tracing::info;

#[derive(Parser)]
#[command(name = "my_app", about = "Platform Manager CLI")]
pub struct Cli {
    #[arg(long, default_value = "http://[::1]:50051")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Execute {
        action: String,
        payload: String,
    },
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let channel = Channel::from_shared(cli.server)?.connect().await?;
    let mut client = ActionServiceClient::new(channel);

    match cli.command {
        Commands::Execute { action, payload } => {
            info!(action = %action, "sending execute request");
            let request = tonic::Request::new(ActionRequest {
                action,
                payload: payload.into_bytes(),
            });
            let response = client.execute(request).await?;
            let resp = response.into_inner();
            if !resp.error.is_empty() {
                eprintln!("Error: {}", resp.error);
            } else {
                let output: serde_json::Value = serde_json::from_slice(&resp.payload)?;
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
        }
    }

    Ok(())
}
