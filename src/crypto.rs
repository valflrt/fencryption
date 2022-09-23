use anyhow::anyhow;
use chacha20poly1305::{
    aead::{stream, Aead},
    KeyInit, XChaCha20Poly1305,
};
use rand::{rngs::OsRng, RngCore};
use ring::digest;
use std::{
    fs::File,
    io::{Read, Write},
};

#[allow(dead_code)]
const SMALL_FILE_NONCE_LEN: usize = 192 / 8;
const LARGE_FILE_NONCE_LEN: usize = 152 / 8;

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
    #[allow(dead_code)]
    pub fn encrypt(&self, plain_data: &[u8]) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
        let nonce = small_file_random_nonce();

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
    #[allow(dead_code)]
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
        let nonce = &encrypted_data[..SMALL_FILE_NONCE_LEN];
        let ciphertext = &encrypted_data[SMALL_FILE_NONCE_LEN..];

        match self.cipher.decrypt(nonce.into(), ciphertext) {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    }

    /// Encrypt a stream from a source io::File and a destination io::File.
    pub fn encrypt_stream(&self, source: &mut File, dest: &mut File) -> anyhow::Result<()> {
        let nonce = large_file_random_nonce();

        let mut stream_cipher =
            stream::EncryptorBE32::from_aead(self.cipher.to_owned(), nonce[..].into());

        const BUFFER_LEN: usize = 500;
        let mut buffer = [0u8; BUFFER_LEN];

        let mut first = true;

        loop {
            let read_len = source.read(&mut buffer)?;

            if read_len == BUFFER_LEN {
                let ciphertext = stream_cipher
                    .encrypt_next(&buffer[..])
                    .map_err(|e| anyhow!(e))?;
                match first {
                    true => dest.write(&[&nonce[..], &ciphertext].concat())?,
                    false => dest.write(&ciphertext)?,
                };
            } else {
                let ciphertext = stream_cipher
                    .encrypt_last(&buffer[..read_len])
                    .map_err(|e| anyhow!(e))?;
                match first {
                    true => dest.write(&[&nonce[..], &ciphertext].concat())?,
                    false => dest.write(&ciphertext)?,
                };
                break;
            }
            if first == true {
                first = !first;
            }
        }

        Ok(())
    }

    /// Decrypt a stream from a source io::File and a destination io::File.
    pub fn decrypt_stream(&self, source: &mut File, dest: &mut File) -> anyhow::Result<()> {
        let mut nonce = [0u8; LARGE_FILE_NONCE_LEN];
        source.read_exact(&mut nonce)?;

        let mut stream_cipher =
            stream::DecryptorBE32::from_aead(self.cipher.to_owned(), &nonce.into());

        const BUFFER_LEN: usize = 500 + 16; // 500 encrypted data and 16 auth tag
        let mut buffer = [0u8; BUFFER_LEN];

        loop {
            let read_len = source.read(&mut buffer)?;

            if read_len == BUFFER_LEN {
                let plaintext = stream_cipher
                    .decrypt_next(&buffer[..])
                    .map_err(|e| anyhow!(e))?;
                dest.write(&plaintext)?;
            } else if read_len == 0 {
                break;
            } else {
                let plaintext = stream_cipher
                    .decrypt_last(&buffer[..read_len])
                    .map_err(|e| anyhow!(e))?;
                dest.write(&plaintext)?;
                break;
            }
        }

        Ok(())
    }
}

fn hash_key(key: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, key).as_ref().to_owned()
}

fn small_file_random_nonce() -> Vec<u8> {
    let mut nonce = [0; SMALL_FILE_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce.to_vec()
}

fn large_file_random_nonce() -> Vec<u8> {
    let mut nonce = [0; LARGE_FILE_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce.to_vec()
}
