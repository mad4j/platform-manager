use clap::{Parser, Subcommand, ValueEnum};
use my_app_transport::{ActionRequest, ActionServiceClient, InfoRequest, InfoServiceClient};
use serde_json::Value;
use tonic::transport::Channel;
use tracing::info;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
}

#[derive(Parser)]
#[command(name = "my_app", about = "Platform Manager CLI")]
pub struct Cli {
    #[arg(long, default_value = "http://[::1]:50051")]
    server: String,

    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Execute {
        action: String,
        payload: String,
    },
    Info,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let channel = Channel::from_shared(cli.server)?.connect().await?;
    let mut action_client = ActionServiceClient::new(channel.clone());
    let mut info_client = InfoServiceClient::new(channel);

    match cli.command {
        Commands::Execute { action, payload } => {
            info!(action = %action, "sending execute request");
            let request = tonic::Request::new(ActionRequest {
                action,
                payload: payload.into_bytes(),
            });
            let response = action_client.execute(request).await?;
            let resp = response.into_inner();
            if !resp.error.is_empty() {
                eprintln!("Error: {}", resp.error);
            } else {
                let output: Value = serde_json::from_slice(&resp.payload)?;
                print_output_value(&output, cli.output)?;
            }
        }
        Commands::Info => {
            info!("sending info request");
            let request = tonic::Request::new(InfoRequest {});
            let response = info_client.info(request).await?;
            let resp = response.into_inner();
            if !resp.error.is_empty() {
                eprintln!("Error: {}", resp.error);
            } else {
                let output = serde_json::json!({
                    "application": resp.application,
                    "endpoints": resp
                        .endpoints
                        .into_iter()
                        .map(|e| serde_json::json!({"name": e.name, "value": e.value}))
                        .collect::<Vec<_>>(),
                    "task_id": resp.task_id,
                });
                print_output_value(&output, cli.output)?;
            }
        }
    }

    Ok(())
}

fn print_output_value(value: &Value, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
        OutputFormat::Table => {
            print_table(value);
        }
    }

    Ok(())
}

fn print_table(value: &Value) {
    match value {
        Value::Object(map) => {
            if map.contains_key("application")
                && map.contains_key("endpoints")
                && map.contains_key("task_id")
            {
                print_info_table(value);
                return;
            }

            let mut rows = Vec::with_capacity(map.len());
            for (k, v) in map {
                rows.push((k.to_string(), render_value(v)));
            }

            print_aligned_two_columns("Key", "Value", &rows);
        }
        Value::Array(values) => {
            let mut rows = Vec::with_capacity(values.len());
            for (idx, v) in values.iter().enumerate() {
                rows.push((idx.to_string(), render_value(v)));
            }

            print_aligned_two_columns("Index", "Value", &rows);
        }
        _ => {
            let rows = vec![("result".to_string(), render_value(value))];
            print_aligned_two_columns("Key", "Value", &rows);
        }
    }
}

fn print_info_table(value: &Value) {
    let application = value
        .get("application")
        .map(render_value)
        .unwrap_or_else(|| "-".to_string());
    let task_id = value
        .get("task_id")
        .map(render_value)
        .unwrap_or_else(|| "-".to_string());

    let field_rows = vec![
        ("application".to_string(), application),
        ("task_id".to_string(), task_id),
    ];
    print_aligned_two_columns("Field", "Value", &field_rows);

    println!();
    let mut endpoint_rows = Vec::new();

    if let Some(Value::Array(endpoints)) = value.get("endpoints") {
        for endpoint in endpoints {
            let name = endpoint
                .get("name")
                .map(render_value)
                .unwrap_or_else(|| "-".to_string());
            let endpoint_value = endpoint
                .get("value")
                .map(render_value)
                .unwrap_or_else(|| "-".to_string());

            endpoint_rows.push((name, endpoint_value));
        }
    }

    print_aligned_two_columns("Endpoint", "Value", &endpoint_rows);
}

fn print_aligned_two_columns(left_header: &str, right_header: &str, rows: &[(String, String)]) {
    let left_width = rows
        .iter()
        .map(|(left, _)| left.chars().count())
        .max()
        .unwrap_or(0)
        .max(left_header.chars().count());

    let right_width = rows
        .iter()
        .map(|(_, right)| right.chars().count())
        .max()
        .unwrap_or(0)
        .max(right_header.chars().count());

    println!(
        "{:<left_width$}  {:<right_width$}",
        left_header,
        right_header,
        left_width = left_width,
        right_width = right_width
    );

    for (left, right) in rows {
        println!(
            "{:<left_width$}  {:<right_width$}",
            left,
            right,
            left_width = left_width,
            right_width = right_width
        );
    }
}

fn render_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}
