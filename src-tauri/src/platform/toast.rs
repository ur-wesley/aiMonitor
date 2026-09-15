use super::shortcut::APP_USER_MODEL_ID;

pub struct WindowsToast;

impl WindowsToast {
    pub fn initialize() {
        super::shortcut::ensure_start_menu_shortcut();
        #[cfg(windows)]
        set_app_user_model_id();
    }

    pub fn try_show(title: &str, body: &str) -> bool {
        #[cfg(windows)]
        {
            show_toast(title, body)
        }
        #[cfg(not(windows))]
        {
            let _ = (title, body);
            false
        }
    }
}

#[cfg(windows)]
fn set_app_user_model_id() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let wide: Vec<u16> = OsStr::new(APP_USER_MODEL_ID)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(windows::core::PCWSTR(wide.as_ptr())) };
}

#[cfg(windows)]
fn show_toast(title: &str, body: &str) -> bool {
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

    let xml = format!(
        "<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual></toast>",
        escape_xml(title),
        escape_xml(body),
    );

    let Ok(doc) = XmlDocument::new() else {
        return false;
    };
    if doc.LoadXml(&windows::core::HSTRING::from(&xml)).is_err() {
        return false;
    }

    let Ok(toast) = ToastNotification::CreateToastNotification(&doc) else {
        return false;
    };

    let Ok(notifier) = ToastNotificationManager::CreateToastNotifierWithId(
        &windows::core::HSTRING::from(APP_USER_MODEL_ID),
    ) else {
        return false;
    };

    notifier.Show(&toast).is_ok()
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
