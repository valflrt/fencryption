use aes_gcm::{
    aead::{Aead, OsRng},
    Aes256Gcm, KeyInit, Nonce,
};
use anyhow::anyhow;
use rand::RngCore;
use ring::digest;

const NONCE_LEN: usize = 12;

pub fn encrypt(key: &[u8], plain_data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let hashed_key = hash_key(key);
    let nonce = Nonce::from(generate_nonce());

    let cipher = match Aes256Gcm::new_from_slice(&hashed_key) {
        Ok(v) => v,
        Err(e) => return Err(anyhow!("Failed to create cipher: {}", e)),
    };

    let encrypted_data = match cipher.encrypt(&nonce, plain_data) {
        Ok(enc) => enc,
        Err(e) => return Err(anyhow!("Failed to encrypt: {}", e)),
    };

    Ok([nonce.to_vec(), encrypted_data].concat())
}

pub fn decrypt(key: &[u8], encrypted_data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let hashed_key = hash_key(key);

    let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LEN]);

    let cipher = match Aes256Gcm::new_from_slice(&hashed_key) {
        Ok(v) => v,
        Err(e) => return Err(anyhow!("Failed to create cipher: {}", e)),
    };

    let decrypted_data = match cipher.decrypt(nonce, &encrypted_data[NONCE_LEN..]) {
        Ok(dec) => dec,
        Err(e) => return Err(anyhow!("Failed to decrypt: {}", e)),
    };

    Ok(decrypted_data)
}

pub fn hash_key(key: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, key).as_ref().to_owned()
}

fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}
