pub const APP_USER_MODEL_ID: &str = "urWesley.aiMonitor";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn ensure_start_menu_shortcut() {
    #[cfg(windows)]
    {
        let shortcut_path = start_menu_shortcut_path();
        let Some(exe) = std::env::current_exe().ok() else {
            return;
        };

        let needs_update = if !shortcut_path.exists() {
            true
        } else {
            shortcut_target(&shortcut_path)
                .map(|target| !paths_equal(&target, &exe))
                .unwrap_or(true)
        };

        if !needs_update {
            return;
        }

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
fn paths_equal(left: &std::path::Path, right: &std::path::Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

#[cfg(windows)]
fn shortcut_target(path: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let ps = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath",
        path.display(),
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if target.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(target))
    }
}

#[cfg(windows)]
fn create_shortcut(path: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let ps = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Description = 'aiMonitor'; $s.Save()",
        path.display(),
        target.display(),
    );
    Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
