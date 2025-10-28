use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64;
use keyring::Entry;
use rand::RngCore;
use std::{path::Path, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum KeyManagerError {
    #[error("Keyring not available: {0}")]
    KeyringUnavailable(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key decode failed: {0}")]
    KeyDecodeFailed(String),
}

pub enum KeyStorage {
    Keychain(Entry),
    Volatile(Arc<Mutex<Option<Key<Aes256Gcm>>>>),
}

pub struct KeyManager {
    storage: KeyStorage,
}

impl KeyManager {
    pub fn new() -> Self {
        match Entry::new("shadowlearn", "encryption_key") {
            Ok(entry) => {
                tracing::info!("✅ Keychain OK - encryption key stored securely");
                Self {
                    storage: KeyStorage::Keychain(entry),
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ Keychain unavailable: {} → using volatile key (no persistence)", e);
                Self {
                    storage: KeyStorage::Volatile(Arc::new(Mutex::new(None))),
                }
            }
        }
    }

    pub fn is_persistent(&self) -> bool {
        matches!(self.storage, KeyStorage::Keychain(_))
    }

    pub async fn get_or_create_key(&self) -> Result<Key<Aes256Gcm>, KeyManagerError> {
        match &self.storage {
            KeyStorage::Keychain(entry) => {
                match entry.get_password() {
                    Ok(b64) => {
                        let bytes = base64::decode(&b64)
                            .map_err(|e| KeyManagerError::KeyDecodeFailed(e.to_string()))?;
                        if bytes.len() != 32 {
                            return Err(KeyManagerError::KeyDecodeFailed("Invalid key length".into()));
                        }
                        Ok(*Key::<Aes256Gcm>::from_slice(&bytes))
                    }
                    Err(_) => {
                        // Generate new key
                        let mut k = [0u8; 32];
                        OsRng.fill_bytes(&mut k);
                        entry.set_password(&base64::encode(k))
                            .map_err(|e| KeyManagerError::KeyringUnavailable(e.to_string()))?;
                        tracing::info!("🔑 Generated new encryption key and stored in keychain");
                        Ok(*Key::<Aes256Gcm>::from_slice(&k))
                    }
                }
            }
            KeyStorage::Volatile(m) => {
                let mut g = m.lock().await;
                if let Some(k) = *g {
                    return Ok(k);
                }
                let mut k = [0u8; 32];
                OsRng.fill_bytes(&mut k);
                let kk = *Key::<Aes256Gcm>::from_slice(&k);
                *g = Some(kk);
                Ok(kk)
            }
        }
    }

    pub fn encrypt_with_key(&self, key: &Key<Aes256Gcm>, data: &[u8]) -> Result<Vec<u8>, KeyManagerError> {
        let cipher = Aes256Gcm::new(key);
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce), data)
            .map_err(|e| KeyManagerError::EncryptionFailed(e.to_string()))?;
        let mut out = Vec::with_capacity(12 + ct.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    pub fn decrypt_with_key(&self, key: &Key<Aes256Gcm>, enc: &[u8]) -> Result<Vec<u8>, KeyManagerError> {
        if enc.len() < 12 {
            return Err(KeyManagerError::DecryptionFailed("Encrypted data too short".into()));
        }
        let cipher = Aes256Gcm::new(key);
        cipher
            .decrypt(Nonce::from_slice(&enc[..12]), &enc[12..])
            .map_err(|e| KeyManagerError::DecryptionFailed(e.to_string()))
    }

    pub async fn rotate_key(&self, dir: &Path) -> Result<(), String> {
        let old = self.get_or_create_key().await.map_err(|e| e.to_string())?;
        let mut nb = [0u8; 32];
        OsRng.fill_bytes(&mut nb);
        let new = Key::<Aes256Gcm>::from_slice(&nb);
        
        let mut rd = tokio::fs::read_dir(dir).await.map_err(|e| e.to_string())?;
        
        while let Some(e) = rd.next_entry().await.map_err(|e| e.to_string())? {
            if e.path().extension().and_then(|s| s.to_str()) == Some("enc") {
                let enc = tokio::fs::read(e.path()).await.map_err(|e| format!("read: {}", e))?;
                let dec = self
                    .decrypt_with_key(&old, &enc)
                    .map_err(|e| format!("decrypt during rotation: {}", e))?;
                let re = self
                    .encrypt_with_key(new, &dec)
                    .map_err(|e| format!("encrypt during rotation: {}", e))?;
                
                // Atomic write
                let tmp = e.path().with_extension("enc.tmp");
                tokio::fs::write(&tmp, &re).await.map_err(|e| format!("write temp: {}", e))?;
                tokio::fs::rename(&tmp, e.path()).await.map_err(|e| format!("rename: {}", e))?;
            }
        }
        
        if let KeyStorage::Keychain(entry) = &self.storage {
            entry.set_password(&base64::encode(nb)).map_err(|e| format!("store new key: {}", e))?;
        }
        
        tracing::info!("✅ Key rotation complete");
        Ok(())
    }
}

#[tauri::command]
pub fn check_keychain_status() -> KeychainStatus {
    let manager = KeyManager::new();
    KeychainStatus {
        available: manager.is_persistent(),
        message: if manager.is_persistent() {
            "🔐 Clé de chiffrement stockée dans le trousseau système (sécurisé)"
                .to_string()
        } else {
            "⚠️ Trousseau non disponible - clé volatile (captures ne persisteront pas au redémarrage)"
                .to_string()
        },
    }
}

#[derive(serde::Serialize)]
pub struct KeychainStatus {
    pub available: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nonces_are_unique() {
        let manager = KeyManager::new();
        let test_data = b"test data";
        let key = manager.get_or_create_key().await.unwrap();

        let mut nonces = Vec::new();

        for _ in 0..10 {
            let encrypted = manager.encrypt_with_key(&key, test_data).unwrap();
            let nonce = &encrypted[..12];
            nonces.push(nonce.to_vec());
        }

        for i in 0..nonces.len() {
            for j in (i + 1)..nonces.len() {
                assert_ne!(nonces[i], nonces[j], "Nonces should be unique");
            }
        }
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let manager = KeyManager::new();
        let key = manager.get_or_create_key().await.unwrap();
        let original = b"sensitive data";

        let encrypted = manager.encrypt_with_key(&key, original).unwrap();
        let decrypted = manager.decrypt_with_key(&key, &encrypted).unwrap();

        assert_eq!(original, decrypted.as_slice());
    }
}
