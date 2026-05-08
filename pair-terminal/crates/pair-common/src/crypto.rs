use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes128Gcm, Nonce,
};
use anyhow::{bail, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const ROTATION_THRESHOLD: u64 = 1 << 20;
const MAX_MESSAGES_PER_KEY: u64 = 2 * (1 << 20);
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 16;

#[derive(Clone)]
pub struct SessionKeys {
    pub bootstrap_key: [u8; KEY_SIZE],
    pub output_key: [u8; KEY_SIZE],
    pub input_key: [u8; KEY_SIZE],
    inner: Mutex<KeysInner>,
}

#[derive(Default)]
struct KeysInner {
    output_iv_count: u64,
    input_iv_count: u64,
    output_messages: u64,
    input_messages: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKeys {
    pub b64_output_key: String,
    pub b64_input_key: String,
    pub iv_count: u64,
    pub max_iv_count: u64,
}

impl SessionKeys {
    pub fn generate() -> Self {
        let mut bootstrap_key = [0u8; KEY_SIZE];
        let mut output_key = [0u8; KEY_SIZE];
        let mut input_key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut bootstrap_key);
        OsRng.fill_bytes(&mut output_key);
        OsRng.fill_bytes(&mut input_key);

        Self {
            bootstrap_key,
            output_key,
            input_key,
            inner: Mutex::new(KeysInner::default()),
        }
    }

    pub fn encrypt_output(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock: {}", e))?;
        inner.output_messages += 1;
        if inner.output_messages > MAX_MESSAGES_PER_KEY {
            bail!("output key exhausted, rotation required");
        }
        let ciphertext = encrypt(&self.output_key, inner.output_iv_count, plaintext)?;
        inner.output_iv_count += 1;
        Ok(ciphertext)
    }

    pub fn decrypt_output(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let mut inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock: {}", e))?;
        let plaintext = decrypt(&self.output_key, inner.output_iv_count, ciphertext)?;
        inner.output_iv_count += 1;
        Ok(plaintext)
    }

    pub fn encrypt_input(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock: {}", e))?;
        inner.input_messages += 1;
        if inner.input_messages > MAX_MESSAGES_PER_KEY {
            bail!("input key exhausted, rotation required");
        }
        let ciphertext = encrypt(&self.input_key, inner.input_iv_count, plaintext)?;
        inner.input_iv_count += 1;
        Ok(ciphertext)
    }

    pub fn decrypt_input(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let mut inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock: {}", e))?;
        let plaintext = decrypt(&self.input_key, inner.input_iv_count, ciphertext)?;
        inner.input_iv_count += 1;
        Ok(plaintext)
    }

    pub fn needs_rotation(&self) -> bool {
        let inner = self.inner.lock().ok();
        inner.map(|i| i.output_messages >= ROTATION_THRESHOLD || i.input_messages >= ROTATION_THRESHOLD).unwrap_or(false)
    }

    pub fn rotate(&self) -> Result<EncryptedKeys> {
        let mut inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock: {}", e))?;

        let mut new_output_key = [0u8; KEY_SIZE];
        let mut new_input_key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut new_output_key);
        OsRng.fill_bytes(&mut new_input_key);

        let encrypted_output = encrypt(&self.bootstrap_key, inner.output_iv_count, &new_output_key)?;
        let encrypted_input = encrypt(&self.bootstrap_key, inner.input_iv_count, &new_input_key)?;

        let mut output_key = [0u8; KEY_SIZE];
        let mut input_key = [0u8; KEY_SIZE];
        output_key.copy_from_slice(&new_output_key);
        input_key.copy_from_slice(&new_input_key);

        let iv_count = inner.output_iv_count;
        inner.output_messages = 0;
        inner.input_messages = 0;
        inner.output_iv_count = 0;
        inner.input_iv_count = 0;

        Ok(EncryptedKeys {
            b64_output_key: base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                &encrypted_output,
            ),
            b64_input_key: base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                &encrypted_input,
            ),
            iv_count: 0,
            max_iv_count: ROTATION_THRESHOLD,
        })
    }

    pub fn bootstrap_key_b64(&self) -> String {
        base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &self.bootstrap_key,
        )
    }

    pub fn extract_keys(bootstrap_key: &[u8], encrypted: &EncryptedKeys) -> Result<Self> {
        let output_key_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &encrypted.b64_output_key,
        )?;
        let input_key_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &encrypted.b64_input_key,
        )?;

        let mut output_key = [0u8; KEY_SIZE];
        let mut input_key = [0u8; KEY_SIZE];
        output_key.copy_from_slice(&decrypt(bootstrap_key, encrypted.iv_count, &output_key_bytes)?);
        input_key.copy_from_slice(&decrypt(bootstrap_key, encrypted.iv_count, &input_key_bytes)?);

        let mut bootstrap = [0u8; KEY_SIZE];
        bootstrap.copy_from_slice(bootstrap_key);

        Ok(Self {
            bootstrap_key: bootstrap,
            output_key,
            input_key,
            inner: Mutex::new(KeysInner {
                output_iv_count: encrypted.iv_count,
                input_iv_count: encrypted.iv_count,
                output_messages: 0,
                input_messages: 0,
            }),
        })
    }
}

fn make_nonce(counter: u64) -> Nonce {
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    nonce_bytes[..8].copy_from_slice(&counter.to_le_bytes());
    Nonce::from_slice(&nonce_bytes).clone()
}

fn encrypt(key: &[u8; KEY_SIZE], iv_counter: u64, plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes128Gcm::new_from_slice(key)?;
    let nonce = make_nonce(iv_counter);
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

fn decrypt(key: &[u8; KEY_SIZE], iv_counter: u64, ciphertext: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE {
        bail!("ciphertext too short");
    }
    let cipher = Aes128Gcm::new_from_slice(key)?;
    let nonce = make_nonce(iv_counter);
    cipher.decrypt(&nonce, &ciphertext[NONCE_SIZE..]).map_err(Into::into)
}

pub fn generate_bootstrap_key() -> [u8; KEY_SIZE] {
    let mut key = [0u8; KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &bytes,
    )
}
