use aes_gcm::{aead::Aead, aes::cipher::InvalidLength, Aes256Gcm, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use ring::digest;
use std::{
    fs::File,
    io::{self, Read, Write},
};

const IV_LEN: usize = 96 / 8; // 12
const TAG_LEN: usize = 128 / 8; // 16
const DEFAULT_BUFFER_LEN: usize = 8192;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidKeyLength(InvalidLength),
    AesError(aes_gcm::Error),
    Io(io::Error),
}

#[derive(Clone)]
pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Creates a new Crypto instance, the given key will be
    /// used for every operation performed.
    pub fn new(key: &[u8]) -> Result<Crypto, ErrorKind> {
        Ok(Crypto {
            cipher: match Aes256Gcm::new_from_slice(&hash_key(key)) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::InvalidKeyLength(e)),
            },
        })
    }

    /// Basic function to encrypt.
    pub fn encrypt_with_nonce(&self, plaintext: &[u8], iv: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        match self.cipher.encrypt(Nonce::from_slice(iv), plaintext) {
            Ok(v) => Ok(v),
            Err(e) => return Err(ErrorKind::AesError(e)),
        }
    }

    /// Basic function to decrypt.
    pub fn decrypt_with_nonce(&self, ciphertext: &[u8], iv: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        match self.cipher.decrypt(Nonce::from_slice(iv), ciphertext) {
            Ok(v) => Ok(v),
            Err(e) => Err(ErrorKind::AesError(e)),
        }
    }

    /// Encrypt a small piece of data.
    ///
    /// Example:
    ///
    /// ```
    /// use fencryption::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// let enc = crypto.encrypt(my_super_secret_message).unwrap();
    ///
    /// assert_ne!(my_super_secret_message, enc);
    /// ```
    pub fn encrypt(&self, plain: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        let iv = &random_iv();

        Ok([iv, self.encrypt_with_nonce(plain, iv)?.as_slice()].concat())
    }

    /// Decrypt a small piece of data.
    ///
    /// Example:
    ///
    /// ```
    /// use fencryption::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// let enc = crypto.encrypt(my_super_secret_message).unwrap();
    /// let dec = crypto.decrypt(&enc).unwrap();
    ///
    /// assert_eq!(my_super_secret_message, dec);
    /// ```
    pub fn decrypt(&self, enc: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        let iv = &enc[..IV_LEN];
        let ciphertext = &enc[IV_LEN..];

        self.decrypt_with_nonce(ciphertext, iv)
    }

    /// Encrypt a stream from a source (io::File) and a
    /// destination (io::File).
    ///
    /// Example:
    ///
    /// ```rust
    /// use std::{
    ///     env,
    ///     fs::{self, File},
    /// };
    /// use fencryption::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// let tmp_dir = env::temp_dir();
    /// let plain_path = tmp_dir.join("encrypt_stream_test");
    /// let enc_path = tmp_dir.join("encrypt_stream_test.enc");
    /// let dec_path = tmp_dir.join("encrypt_stream_test.dec");
    ///
    /// fs::remove_file(&plain_path).ok();
    /// fs::remove_file(&enc_path).ok();
    /// fs::remove_file(&dec_path).ok();
    ///
    /// fs::write(&plain_path, my_super_secret_message).unwrap();
    ///
    /// let mut plain = File::open(&plain_path).unwrap();
    /// let mut enc = File::create(&enc_path).unwrap();
    ///
    /// crypto.encrypt_stream(&mut plain, &mut enc).unwrap();
    /// ```
    pub fn encrypt_stream(&self, source: &mut File, dest: &mut File) -> Result<(), ErrorKind> {
        let iv = random_iv();

        const BUFFER_LEN: usize = DEFAULT_BUFFER_LEN;
        let mut buffer = [0u8; BUFFER_LEN];

        if let Err(e) = dest.write_all(&iv) {
            return Err(ErrorKind::Io(e));
        };

        loop {
            let read_len = match source.read(&mut buffer) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::Io(e)),
            };

            if let Err(e) = dest.write(&self.encrypt_with_nonce(&buffer[..read_len], &iv)?) {
                return Err(ErrorKind::Io(e));
            };

            // Stops when the loop reached the end of the file
            if read_len != BUFFER_LEN {
                break;
            };
        }

        Ok(())
    }

    /// Decrypt a stream from a source (io::File) and a
    /// destination (io::File).
    ///
    /// Example:
    ///
    /// ```rust
    /// use std::{
    ///     env,
    ///     fs::{self, File},
    /// };
    /// use fencryption::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// let tmp_dir = env::temp_dir();
    /// let plain_path = tmp_dir.join("encrypt_stream_test");
    /// let enc_path = tmp_dir.join("encrypt_stream_test.enc");
    /// let dec_path = tmp_dir.join("encrypt_stream_test.dec");
    ///
    /// fs::remove_file(&plain_path).ok();
    /// fs::remove_file(&enc_path).ok();
    /// fs::remove_file(&dec_path).ok();
    ///
    /// fs::write(&plain_path, my_super_secret_message).unwrap();
    ///
    /// let mut plain = File::open(&plain_path).unwrap();
    /// let mut enc = File::create(&enc_path).unwrap();
    ///
    /// crypto.encrypt_stream(&mut plain, &mut enc).unwrap();
    ///
    /// let mut enc = File::open(&enc_path).unwrap();
    /// let mut dec = File::create(&dec_path).unwrap();
    ///
    /// crypto.decrypt_stream(&mut enc, &mut dec).unwrap();
    ///
    /// assert_eq!(my_super_secret_message[..], fs::read(&dec_path).unwrap());
    /// ```
    pub fn decrypt_stream(&self, source: &mut File, dest: &mut File) -> Result<(), ErrorKind> {
        const BUFFER_LEN: usize = DEFAULT_BUFFER_LEN + TAG_LEN; // ciphertext (500) + auth tag (16)
        let mut buffer = [0u8; BUFFER_LEN];

        let mut iv = [0u8; IV_LEN];
        if let Err(e) = source.read_exact(&mut iv) {
            return Err(ErrorKind::Io(e));
        };

        loop {
            let read_len = match source.read(&mut buffer) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::Io(e)),
            };

            if let Err(e) = dest.write(&self.decrypt_with_nonce(&buffer[..read_len], &iv)?) {
                return Err(ErrorKind::Io(e));
            };

            // Stops when the loop reached the end of the file.
            if read_len != BUFFER_LEN {
                break;
            };
        }

        Ok(())
    }
}

fn hash_key(key: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, key).as_ref().to_owned()
}

fn random_iv() -> Vec<u8> {
    let mut iv = [0; IV_LEN];
    OsRng.fill_bytes(&mut iv);
    iv.to_vec()
}
