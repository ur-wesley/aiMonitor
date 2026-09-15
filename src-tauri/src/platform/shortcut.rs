pub const APP_USER_MODEL_ID: &str = "urWesley.aiMonitor";

pub fn ensure_start_menu_shortcut() {
    #[cfg(windows)]
    {
        let shortcut_path = start_menu_shortcut_path();
        if shortcut_path.exists() {
            return;
        }

        let Some(exe) = std::env::current_exe().ok() else {
            return;
        };

        if let Some(parent) = shortcut_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let _ = create_shortcut(&shortcut_path, &exe);
    }
}

#[cfg(windows)]
fn start_menu_shortcut_path() -> std::path::PathBuf {
    let start_menu = std::env::var("APPDATA").map_or_else(|_| std::path::PathBuf::from("."), std::path::PathBuf::from);
    start_menu.join("Microsoft").join("Windows").join("Start Menu").join("Programs").join("aiMonitor.lnk")
}

#[cfg(windows)]
fn create_shortcut(path: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    use std::process::Command;

    let ps = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Description = 'aiMonitor'; $s.Save()",
        path.display(),
        target.display(),
    );
    Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
