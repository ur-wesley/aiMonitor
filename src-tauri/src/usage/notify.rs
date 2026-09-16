use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::models::{AppSettings, UsageSnapshot, UsageWindowMetric};
use crate::platform::toast::WindowsToast;

pub struct LowUsageNotifier {
    settings: Arc<Mutex<AppSettings>>,
    notified: Mutex<HashSet<String>>,
}

impl LowUsageNotifier {
    pub fn new(settings: Arc<Mutex<AppSettings>>) -> Self {
        Self {
            settings,
            notified: Mutex::new(HashSet::new()),
        }
    }

    pub fn evaluate(&self, snapshot: &UsageSnapshot) {
        if !self.settings.lock().low_usage_notifications_enabled {
            return;
        }

        for provider in &snapshot.providers {
            if provider.error.is_some() || provider.windows.is_empty() {
                continue;
            }

            let Some(window) = find_most_depleted(&provider.windows) else {
                continue;
            };
            let remaining = window.remaining_percent();
            let key = build_key(&provider.provider_id, window);

            if remaining > 10.0 {
                self.notified.lock().remove(&key);
                continue;
            }

            if self.notified.lock().contains(&key) {
                continue;
            }

            let title = format!("{} quota low", provider.display_name);
            let body = format!("{}: {:.0}% left", window.label, remaining);
            if WindowsToast::try_show(&title, &body) {
                self.notified.lock().insert(key);
            }
        }
    }
}

fn find_most_depleted(windows: &[UsageWindowMetric]) -> Option<&UsageWindowMetric> {
    windows
        .iter()
        .min_by(|a, b| {
            a.remaining_percent()
                .partial_cmp(&b.remaining_percent())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn build_key(provider_id: &str, window: &UsageWindowMetric) -> String {
    let reset = window
        .resets_at
        .map_or(0, |r| r.timestamp());
    format!("{}|{}|{}", provider_id, window.label, reset)
}
