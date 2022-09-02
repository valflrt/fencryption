use aes_gcm::{aead::OsRng, aes::cipher::InvalidLength, AeadInPlace, Aes256Gcm, KeyInit, Nonce};
use anyhow::anyhow;
use rand::RngCore;
use ring::digest;

const NONCE_LEN: usize = 96 / 8;
const TAG_LEN: usize = 128 / 8;

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Creates a new Crypto instance, the given key will be
    /// used for every operation performed.
    pub fn new(key: &[u8]) -> Result<Crypto, InvalidLength> {
        let cipher = match Aes256Gcm::new_from_slice(&hash_key(key)) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(Crypto { cipher })
    }

    /// Encrypt a small piece of data.
    ///
    /// ### Example
    ///
    /// ```
    /// use fencryption::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = match Crypto::new(my_super_key) {
    ///     Ok(v) => v,
    ///     Err(e) => panic!("Failed to create cipher: {}", e),
    /// };
    ///
    /// let enc_data = match crypto.encrypt(my_super_secret_message) {
    ///     Ok(enc) => enc,
    ///     Err(e) => panic!("Failed to encrypt: {}", e),
    /// };
    /// ```
    pub fn encrypt(&self, plain_data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let nonce = Nonce::from(randomize_nonce());
        let mut buffer = Vec::with_capacity(plain_data.len() + TAG_LEN);
        buffer.extend_from_slice(plain_data);

        if let Err(e) = self.cipher.encrypt_in_place(&nonce, &[], &mut buffer) {
            return Err(anyhow!(e));
        };

        let result = [nonce.to_vec(), buffer].concat();

        Ok(result)
    }

    /// Decrypt a small encrypted piece of data.
    ///
    /// Example:
    /// ```
    /// use fencryption::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = match Crypto::new(my_super_key) {
    ///     Ok(v) => v,
    ///     Err(e) => panic!("Failed to create cipher: {}", e),
    /// };
    ///
    /// let enc_data = match crypto.encrypt(&my_super_secret_message) {
    ///     Ok(enc) => enc,
    ///     Err(e) => panic!("Failed to encrypt: {}", e),
    /// };
    ///
    /// let dec_data = match crypto.decrypt(&enc_data) {
    ///     Ok(dec) => dec,
    ///     Err(e) => panic!("Failed to decrypt: {}", e),
    /// };
    /// ```
    pub fn decrypt(&self, encrypted_data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LEN]);
        let mut buffer = encrypted_data[NONCE_LEN..].to_vec();

        if let Err(e) = self.cipher.decrypt_in_place(nonce, &[], &mut buffer) {
            return Err(anyhow!(e));
        };

        Ok(buffer)
    }
}

fn hash_key(key: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, key).as_ref().to_owned()
}

fn randomize_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}
