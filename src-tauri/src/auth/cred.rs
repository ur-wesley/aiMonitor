use base64::Engine;

#[cfg(windows)]
pub fn read_antigravity_token() -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Security::Credentials::{CredFree, CredReadW, CRED_TYPE_GENERIC};

    let target: Vec<u16> = OsStr::new("gemini:antigravity")
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut credential_ptr = std::ptr::null_mut();
    let ok = unsafe {
        CredReadW(
            windows::core::PCWSTR(target.as_ptr()),
            CRED_TYPE_GENERIC,
            Some(0),
            &raw mut credential_ptr,
        )
    };

    if ok.is_err() || credential_ptr.is_null() {
        return None;
    }

    let credential = unsafe { &*credential_ptr.cast_const() };
    if credential.CredentialBlobSize == 0 || credential.CredentialBlob.is_null() {
        unsafe { CredFree(credential_ptr as _) };
        return None;
    }

    let bytes = unsafe {
        std::slice::from_raw_parts(
            credential.CredentialBlob,
            credential.CredentialBlobSize as usize,
        )
    };
    let raw = String::from_utf8_lossy(bytes).to_string();
    unsafe { CredFree(credential_ptr as _) };

    if let Some(encoded) = raw.strip_prefix("go-keyring-base64:") {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
            return String::from_utf8(decoded).ok();
        }
    }

    Some(raw)
}

#[cfg(not(windows))]
pub fn read_antigravity_token() -> Option<String> {
    None
}
