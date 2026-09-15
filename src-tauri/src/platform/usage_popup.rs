use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tauri::WebviewWindow;

use crate::platform::placement::{self, WorkAreaBounds};

pub struct UsagePopupGuard {
    hide_allowed_after: Mutex<Option<Instant>>,
    anchor: Mutex<Option<(i32, i32)>>,
    work_area: Mutex<Option<WorkAreaBounds>>,
}

impl UsagePopupGuard {
    pub const fn new() -> Self {
        Self {
            hide_allowed_after: Mutex::new(None),
            anchor: Mutex::new(None),
            work_area: Mutex::new(None),
        }
    }

    pub fn arm_show(&self) {
        *self.hide_allowed_after.lock() = Some(Instant::now() + Duration::from_millis(400));
    }

    pub fn may_hide(&self) -> bool {
        self.hide_allowed_after
            .lock()
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(true)
    }

    pub fn place_at_cursor(&self, window: &WebviewWindow) -> Result<(), String> {
        let anchor = placement::cursor_anchor(window)?;
        let work_area = placement::work_area_for_point(window, f64::from(anchor.0), f64::from(anchor.1))?;
        *self.anchor.lock() = Some(anchor);
        *self.work_area.lock() = Some(work_area);
        placement::place_window(window, anchor, work_area)
    }

    pub fn place_from_anchor(&self, window: &WebviewWindow) -> Result<(), String> {
        let anchor = (*self.anchor.lock()).ok_or_else(|| "usage popup anchor missing".to_string())?;
        let work_area = (*self.work_area.lock()).ok_or_else(|| "usage popup work area missing".to_string())?;
        placement::place_window(window, anchor, work_area)
    }
}
