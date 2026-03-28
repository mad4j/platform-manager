use std::sync::Arc;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use super::launched_apps::LaunchedApps;
use crate::errors::AppError;
use crate::models::{DeployRequest, DeployResponse};

pub struct DeployAction {
    launched_apps: Arc<LaunchedApps>,
}

impl DeployAction {
    pub fn new(launched_apps: Arc<LaunchedApps>) -> Self {
        Self { launched_apps }
    }

    pub fn deploy(&self, input: Vec<u8>) -> Result<Vec<u8>, AppError> {
        let req: DeployRequest =
            serde_json::from_slice(&input).map_err(|_| AppError::InvalidPayload)?;

        let app_name = req.application.trim();
        if app_name.is_empty() {
            return Err(AppError::InvalidPayload);
        }

        let listen = req.listen.as_deref().map(str::trim).filter(|value| !value.is_empty());

        let app_url = match req.url.as_deref().map(str::trim).filter(|url| !url.is_empty()) {
            Some(url) => Self::normalize_url(url),
            None => {
                if let Some(listen_addr) = listen {
                    format!("http://{listen_addr}")
                } else {
                    format!("http://localhost/{app_name}")
                }
            }
        };

        if let Some(listen_addr) = listen {
            let mut child = Command::new(app_name)
                .arg("--listen")
                .arg(Self::to_listen_arg(listen_addr))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| AppError::Execution(format!("failed to spawn '{app_name}': {e}")))?;

            if let Err(e) = Self::probe_info_with_retry(&app_url) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        }

        self.launched_apps
            .record(app_name.to_string(), app_url.clone());

        let resp = DeployResponse {
            application: app_name.to_string(),
            url: app_url,
            status: "deployed".to_string(),
            message: format!("application '{app_name}' deployed by deploy"),
        };

        serde_json::to_vec(&resp).map_err(|e| AppError::Execution(e.to_string()))
    }

    fn normalize_url(url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("http://{url}")
        }
    }

    fn to_listen_arg(listen: &str) -> &str {
        listen
            .strip_prefix("http://")
            .or_else(|| listen.strip_prefix("https://"))
            .unwrap_or(listen)
    }

    fn probe_info_with_retry(server_url: &str) -> Result<(), AppError> {
        const MAX_ATTEMPTS: usize = 10;
        const RETRY_DELAY: Duration = Duration::from_millis(250);

        let cli_path = Self::resolve_cli_path()?;
        let mut last_error = String::new();

        for attempt in 1..=MAX_ATTEMPTS {
            let output = Command::new(&cli_path)
                .arg("--server")
                .arg(server_url)
                .arg("--output")
                .arg("json")
                .arg("info")
                .output();

            match output {
                Ok(result) if result.status.success() => return Ok(()),
                Ok(result) => {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    last_error = if stderr.trim().is_empty() {
                        format!("info command exited with {}", result.status)
                    } else {
                        stderr.trim().to_string()
                    };
                }
                Err(e) => {
                    last_error = e.to_string();
                }
            }

            if attempt < MAX_ATTEMPTS {
                thread::sleep(RETRY_DELAY);
            }
        }

        Err(AppError::Execution(format!(
            "failed to validate deployment via info command against '{server_url}': {last_error}"
        )))
    }

    fn resolve_cli_path() -> Result<String, AppError> {
        if let Ok(path) = std::env::var("PLATFORM_MANAGER_CLI_PATH") {
            let path = path.trim();
            if !path.is_empty() {
                return Ok(path.to_string());
            }
        }

        let exe = std::env::current_exe()
            .map_err(|e| AppError::Execution(format!("failed to resolve current executable: {e}")))?;

        let Some(dir) = exe.parent() else {
            return Err(AppError::Execution(
                "failed to resolve executable directory for info probe".to_string(),
            ));
        };

        let cli_name = if cfg!(windows) { "cli.exe" } else { "cli" };
        let cli_path = dir.join(cli_name);
        if cli_path.is_file() {
            Ok(cli_path.to_string_lossy().to_string())
        } else {
            Err(AppError::Execution(format!(
                "unable to find '{cli_name}' for info probe. Set PLATFORM_MANAGER_CLI_PATH to override"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::models::ApplicationAccess;

    #[test]
    fn test_deploy_success() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
            application: "platform-manager".to_string(),
            url: "http://localhost:50051".to_string(),
        }]));
        let action = DeployAction::new(Arc::clone(&launched_apps));

        let input = serde_json::to_vec(&DeployRequest {
            application: "billing-api".to_string(),
            url: Some("https://billing.example.com".to_string()),
            listen: None,
        })
        .unwrap();

        let output = action.deploy(input).unwrap();
        let resp: DeployResponse = serde_json::from_slice(&output).unwrap();

        assert_eq!(resp.application, "billing-api");
        assert_eq!(resp.url, "https://billing.example.com");
        assert_eq!(resp.status, "deployed");
        assert!(resp.message.contains("billing-api"));
        assert!(launched_apps.list().contains(&ApplicationAccess {
            application: "billing-api".to_string(),
            url: "https://billing.example.com".to_string(),
        }));
    }

    #[test]
    fn test_deploy_rejects_invalid_payload() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = DeployAction::new(launched_apps);

        let result = action.deploy(b"not json".to_vec());
        assert!(matches!(result, Err(AppError::InvalidPayload)));
    }

    #[test]
    fn test_deploy_rejects_empty_application() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = DeployAction::new(launched_apps);

        let input = serde_json::to_vec(&DeployRequest {
            application: "   ".to_string(),
            url: None,
            listen: None,
        })
        .unwrap();

        let result = action.deploy(input);
        assert!(matches!(result, Err(AppError::InvalidPayload)));
    }

    #[test]
    fn test_deploy_generates_default_url() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = DeployAction::new(launched_apps);

        let input = serde_json::to_vec(&DeployRequest {
            application: "orders-api".to_string(),
            url: None,
            listen: None,
        })
        .unwrap();

        let output = action.deploy(input).unwrap();
        let resp: DeployResponse = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.url, "http://localhost/orders-api");
    }

    #[test]
    fn test_deploy_normalizes_url_without_scheme() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = DeployAction::new(launched_apps);

        let input = serde_json::to_vec(&DeployRequest {
            application: "orders-api".to_string(),
            url: Some("127.0.0.1:50051".to_string()),
            listen: None,
        })
        .unwrap();

        let output = action.deploy(input).unwrap();
        let resp: DeployResponse = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.url, "http://127.0.0.1:50051");
        assert!(resp.message.contains("deployed by deploy"));
    }
}
