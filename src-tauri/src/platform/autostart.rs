const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "aiMonitor";

#[cfg(windows)]
fn encode_wide_nul(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

#[cfg(windows)]
fn encode_reg_sz(value: &str) -> Vec<u8> {
    encode_wide_nul(value)
        .iter()
        .flat_map(|unit| unit.to_le_bytes())
        .collect()
}

#[cfg(windows)]
fn quoted_exe_path() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    Ok(format!("\"{}\"", exe.display()))
}

#[cfg(windows)]
fn win32_error_message(error: windows::Win32::Foundation::WIN32_ERROR) -> String {
    format!("Win32 error {error:?}")
}

#[cfg(windows)]
fn open_run_key(
    access: windows::Win32::System::Registry::REG_SAM_FLAGS,
) -> Result<windows::Win32::System::Registry::HKEY, String> {
    use windows::Win32::System::Registry::{RegOpenKeyExW, HKEY_CURRENT_USER};

    let subkey = encode_wide_nul(RUN_KEY);
    let mut key = windows::Win32::System::Registry::HKEY::default();
    let error = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            windows::core::PCWSTR(subkey.as_ptr()),
            Some(0),
            access,
            &raw mut key,
        )
    };
    if error.is_err() {
        return Err(win32_error_message(error));
    }
    Ok(key)
}

#[cfg(windows)]
fn run_value_exists(value_name: &str) -> bool {
    use windows::Win32::System::Registry::{
        RegCloseKey, RegQueryValueExW, KEY_QUERY_VALUE, REG_SZ,
    };

    let Ok(key) = open_run_key(KEY_QUERY_VALUE) else {
        return false;
    };

    let value_name = encode_wide_nul(value_name);
    let mut data_type = REG_SZ;
    let mut data_size = 0u32;
    let result = unsafe {
        RegQueryValueExW(
            key,
            windows::core::PCWSTR(value_name.as_ptr()),
            None,
            Some(&raw mut data_type),
            None,
            Some(&raw mut data_size),
        )
    };

    let _ = unsafe { RegCloseKey(key) };
    result.is_ok()
}

#[cfg(windows)]
fn set_run_value(value_name: &str, command: &str) -> Result<(), String> {
    use windows::Win32::System::Registry::{
        RegCloseKey, RegSetValueExW, KEY_SET_VALUE, REG_SZ,
    };

    let key = open_run_key(KEY_SET_VALUE)?;
    let value_name = encode_wide_nul(value_name);
    let data = encode_reg_sz(command);
    let error = unsafe {
        RegSetValueExW(
            key,
            windows::core::PCWSTR(value_name.as_ptr()),
            None,
            REG_SZ,
            Some(data.as_slice()),
        )
    };
    let result = if error.is_err() {
        Err(win32_error_message(error))
    } else {
        Ok(())
    };
    let _ = unsafe { RegCloseKey(key) };
    result
}

#[cfg(windows)]
fn delete_run_value(value_name: &str) -> Result<(), String> {
    use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    use windows::Win32::System::Registry::{RegCloseKey, RegDeleteValueW, KEY_SET_VALUE};

    let key = open_run_key(KEY_SET_VALUE)?;
    let value_name = encode_wide_nul(value_name);
    let error = unsafe { RegDeleteValueW(key, windows::core::PCWSTR(value_name.as_ptr())) };
    let result = if error.is_ok() || error == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(win32_error_message(error))
    };
    let _ = unsafe { RegCloseKey(key) };
    result
}

#[cfg(windows)]
pub fn is_enabled() -> bool {
    run_value_exists(VALUE_NAME)
}

#[cfg(not(windows))]
pub fn is_enabled() -> bool {
    false
}

#[cfg(windows)]
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        let command = quoted_exe_path()?;
        set_run_value(VALUE_NAME, &command)
    } else {
        delete_run_value(VALUE_NAME)
    }
}

#[cfg(not(windows))]
pub fn set_enabled(_enabled: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::{delete_run_value, encode_reg_sz, quoted_exe_path, run_value_exists, set_run_value};

    #[test]
    #[cfg(windows)]
    fn encode_reg_sz_includes_nul_terminator() {
        let bytes = encode_reg_sz("test");
        assert_eq!(bytes.len(), 5 * 2);
        assert_eq!(bytes[bytes.len() - 2], 0);
        assert_eq!(bytes[bytes.len() - 1], 0);
    }

    #[test]
    #[cfg(windows)]
    fn quoted_exe_path_wraps_in_quotes() {
        let path = quoted_exe_path().expect("current_exe");
        assert!(path.starts_with('"'));
        assert!(path.ends_with('"'));
        assert!(path.len() > 2);
    }

    #[test]
    #[cfg(windows)]
    fn registry_roundtrip_under_test_value() {
        const TEST_NAME: &str = "aiMonitor.test.autostart";

        let _ = delete_run_value(TEST_NAME);

        let command = quoted_exe_path().expect("current_exe");
        set_run_value(TEST_NAME, &command).expect("set");
        assert!(run_value_exists(TEST_NAME));

        delete_run_value(TEST_NAME).expect("delete");
        assert!(!run_value_exists(TEST_NAME));
    }
}
