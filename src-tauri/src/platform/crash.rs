use std::fs::OpenOptions;
use std::io::Write;

use crate::paths::crash_log_path;

pub fn register_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let _ = write_crash_log(&format!("panic: {info}"));
    }));
}

pub fn write_crash_log(message: &str) -> std::io::Result<()> {
    let path = crash_log_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{} {}", chrono::Utc::now().to_rfc3339(), message)?;
    Ok(())
}
