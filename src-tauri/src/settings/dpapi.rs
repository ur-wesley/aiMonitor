#[cfg(windows)]
pub fn protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB};

    let len = u32::try_from(plain.len()).map_err(|_| "data too large for DPAPI".to_string())?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: len,
        pbData: plain.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();

    let ok = unsafe {
        CryptProtectData(
            &raw const input,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &raw mut output,
        )
    };

    if ok.is_err() {
        return Err("DPAPI protect failed".into());
    }

    let protected = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe { windows::Win32::Foundation::LocalFree(Some(HLOCAL(output.pbData.cast()))) };
    Ok(protected)
}

#[cfg(windows)]
pub fn unprotect(encrypted: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB};

    let len = u32::try_from(encrypted.len()).map_err(|_| "data too large for DPAPI".to_string())?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: len,
        pbData: encrypted.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();

    let ok = unsafe {
        CryptUnprotectData(
            &raw const input,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &raw mut output,
        )
    };

    if ok.is_err() {
        return Err("DPAPI unprotect failed".into());
    }

    let plain = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe { windows::Win32::Foundation::LocalFree(Some(HLOCAL(output.pbData.cast()))) };
    Ok(plain)
}

#[cfg(not(windows))]
pub fn protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    Ok(plain.to_vec())
}

#[cfg(not(windows))]
pub fn unprotect(encrypted: &[u8]) -> Result<Vec<u8>, String> {
    Ok(encrypted.to_vec())
}
