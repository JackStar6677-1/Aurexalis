//! Descifrado Chromium en Linux (libsecret / keyring + fallback V10 "peanuts").

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;

use crate::ImporterError;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

const SALT: &[u8] = b"saltysalt";
const ITERATIONS: u32 = 1;
const KEY_LEN: usize = 16;
const IV: [u8; 16] = [b' '; 16];

/// Intenta descifrar la clave AES de `Local State` (`os_crypt.encrypted_key`).
pub fn decrypt_linux_encrypted_key(encrypted_key: &[u8]) -> Result<Vec<u8>, ImporterError> {
    if encrypted_key.is_empty() {
        return Err(ImporterError::Crypto(
            "encrypted_key vacio en Local State".to_owned(),
        ));
    }

    let mut passwords = linux_keyring_passwords();
    passwords.push("peanuts".to_owned());

    let mut last_err = ImporterError::UnsupportedDecryption(
        "no se pudo descifrar con libsecret/keyring ni fallback V10",
    );

    for password in passwords {
        for prefix in [b"v11".as_slice(), b"v10".as_slice()] {
            if !encrypted_key.starts_with(prefix) {
                continue;
            }
            match decrypt_with_password(encrypted_key, prefix, password.as_bytes()) {
                Ok(key) if key.len() == 32 => return Ok(key),
                Ok(_) => {
                    last_err = ImporterError::Crypto(
                        "clave Chromium descifrada con longitud inesperada".to_owned(),
                    );
                }
                Err(error) => last_err = error,
            }
        }
    }

    Err(last_err)
}

fn decrypt_with_password(
    encrypted_key: &[u8],
    prefix: &[u8],
    password: &[u8],
) -> Result<Vec<u8>, ImporterError> {
    let raw = encrypted_key
        .get(prefix.len()..)
        .ok_or_else(|| ImporterError::Crypto("blob encrypted_key demasiado corto".to_owned()))?;

    let mut derived = [0_u8; KEY_LEN];
    pbkdf2_hmac::<Sha1>(password, SALT, ITERATIONS, &mut derived);

    let cipher = Aes128CbcDec::new_from_slices(&derived, &IV)
        .map_err(|error| ImporterError::Crypto(error.to_string()))?;
    let mut buffer = raw.to_vec();
    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|error| ImporterError::Crypto(format!("AES-CBC: {error:?}")))?;
    Ok(plaintext.to_vec())
}

/// Contraseñas conocidas en GNOME Keyring / KWallet (Chrome, Chromium, Brave, Opera).
fn linux_keyring_passwords() -> Vec<String> {
    const CANDIDATES: [(&str, &str); 5] = [
        ("chrome", "Chrome Safe Storage"),
        ("chromium", "Chromium Safe Storage"),
        ("Brave", "Brave Safe Storage"),
        ("opera", "Opera Safe Storage"),
        ("google-chrome", "Chrome Safe Storage"),
    ];

    let mut found = Vec::new();
    for (service, user) in CANDIDATES {
        if let Ok(entry) = keyring::Entry::new(service, user) {
            if let Ok(password) = entry.get_password() {
                if !password.is_empty() && !found.iter().any(|p| p == &password) {
                    found.push(password);
                }
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_blob() {
        let err = decrypt_linux_encrypted_key(&[]).expect_err("empty");
        assert!(matches!(err, ImporterError::Crypto(_)));
    }
}
