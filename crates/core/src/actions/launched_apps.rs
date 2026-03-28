use std::sync::Mutex;

use crate::models::ApplicationAccess;

pub struct LaunchedApps {
    apps: Mutex<Vec<ApplicationAccess>>,
}

impl LaunchedApps {
    pub fn new(initial_apps: Vec<ApplicationAccess>) -> Self {
        Self {
            apps: Mutex::new(initial_apps),
        }
    }

    pub fn record(&self, app_name: String, app_url: String) {
        let mut apps = self.apps.lock().expect("launched apps lock poisoned");
        if let Some(existing) = apps.iter_mut().find(|item| item.application == app_name) {
            existing.url = app_url;
            return;
        }

        apps.push(ApplicationAccess {
            application: app_name,
            url: app_url,
        });
    }

    pub fn list(&self) -> Vec<ApplicationAccess> {
        let apps = self.apps.lock().expect("launched apps lock poisoned");
        apps.clone()
    }
}
