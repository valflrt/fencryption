use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};
use rand::{rngs::OsRng, RngCore};
use ring::digest;

const NONCE_LEN: usize = 24;
// const TAG_LEN: usize = 128 / 8;

pub struct Crypto {
    cipher: XChaCha20Poly1305,
}

impl Crypto {
    /// Creates a new Crypto instance, the given key will be
    /// used for every operation performed.
    pub fn new(key: &[u8]) -> Crypto {
        Crypto {
            cipher: XChaCha20Poly1305::new(hash_key(key)[..].into()),
        }
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
    /// let crypto = Crypto::new(my_super_key);
    ///
    /// let enc_data = match crypto.encrypt(my_super_secret_message) {
    ///     Ok(enc) => enc,
    ///     Err(e) => panic!("Failed to encrypt: {}", e),
    /// };
    /// ```
    pub fn encrypt(&self, plain_data: &[u8]) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
        let nonce = random_nonce();

        match self.cipher.encrypt(nonce[..].into(), plain_data) {
            Ok(v) => Ok([nonce.to_vec(), v].concat()),
            Err(e) => return Err(e),
        }
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
    /// let crypto = Crypto::new(my_super_key);
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
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
        let nonce = &encrypted_data[..NONCE_LEN];
        let ciphertext = &encrypted_data[NONCE_LEN..];

        match self.cipher.decrypt(nonce.into(), ciphertext) {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    }
}

fn hash_key(key: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, key).as_ref().to_owned()
}

fn random_nonce() -> Vec<u8> {
    let mut nonce = [0; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce.to_vec()
}
