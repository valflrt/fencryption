use aes_gcm::{aead::Aead, aes::cipher::InvalidLength, Aes256Gcm, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use ring::digest;
use std::{
    fs::File,
    io::{self, Read, Write},
};

use crate::constants::DEFAULT_BUFFER_LEN;

const IV_LEN: usize = 96 / 8; // 12
const TAG_LEN: usize = 128 / 8; // 16

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
    pub fn new<K>(key: K) -> Result<Crypto, ErrorKind>
    where
        K: AsRef<[u8]>,
    {
        Ok(Crypto {
            cipher: match Aes256Gcm::new_from_slice(&hash_key(key.as_ref())) {
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
    /// use fencryption_lib::crypto::Crypto;
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
    pub fn encrypt<P>(&self, plain: P) -> Result<Vec<u8>, ErrorKind>
    where
        P: AsRef<[u8]>,
    {
        let iv = &random_iv();
        Ok([iv, self.encrypt_with_nonce(plain.as_ref(), iv)?.as_slice()].concat())
    }

    /// Decrypt a small piece of data.
    ///
    /// Example:
    ///
    /// ```
    /// use fencryption_lib::crypto::Crypto;
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
    pub fn decrypt<E>(&self, enc: E) -> Result<Vec<u8>, ErrorKind>
    where
        E: AsRef<[u8]>,
    {
        let iv = &enc.as_ref()[..IV_LEN];
        let ciphertext = &enc.as_ref()[IV_LEN..];

        self.decrypt_with_nonce(ciphertext, iv)
    }

    /// Encrypt a stream from a source and a destination
    /// (both [std::fs::File]).
    ///
    /// Example:
    ///
    /// ```rust
    /// use fencryption_lib::crypto::Crypto;
    /// use fencryption_lib::test::util::TmpDir;
    ///
    /// let my_super_key = b"this_is_super_secure";
    /// let my_super_secret_message = b"hello :)";
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// // Creates a temporary directory
    /// let tmp_dir = TmpDir::new().unwrap();
    ///
    /// // tmp_dir.write_file is akin to std::fs::write
    /// tmp_dir
    ///     .write_file("plain", my_super_secret_message)
    ///     .unwrap();
    /// crypto
    ///     .encrypt_stream(
    ///         // tmp_dir.open_file is akin to std::fs::File::open
    ///         &mut tmp_dir.open_file("plain").unwrap(),
    ///         // tmp_dir.create_file is akin to std::fs::File::create
    ///         &mut tmp_dir.create_file("enc").unwrap(),
    ///     )
    ///     .unwrap();
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

    /// Decrypt a stream from a source and a destination
    /// (both [std::fs::File]).
    ///
    /// Example:
    ///
    /// ```rust
    /// use fencryption_lib::crypto::Crypto;
    /// use fencryption_lib::test::util::TmpDir;
    ///
    /// let my_super_key = b"this_is_super_secure";
    /// let my_super_secret_message = b"hello :)";
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// // Creates a temporary directory
    /// let tmp_dir = TmpDir::new().unwrap();
    ///
    /// // tmp_dir.write_file is akin to std::fs::write
    /// tmp_dir
    ///     .write_file("plain", my_super_secret_message)
    ///     .unwrap();
    /// crypto
    ///     .encrypt_stream(
    ///         // tmp_dir.open_file is akin to std::fs::File::open
    ///         &mut tmp_dir.open_file("plain").unwrap(),
    ///         // tmp_dir.create_file is akin to std::fs::File::create
    ///         &mut tmp_dir.create_file("enc").unwrap(),
    ///     )
    ///     .unwrap();
    ///
    /// crypto
    ///     .decrypt_stream(
    ///         // tmp_dir.open_file is akin to std::fs::File::open
    ///         &mut tmp_dir.open_file("enc").unwrap(),
    ///         // tmp_dir.create_file is akin to std::fs::File::create
    ///         &mut tmp_dir.create_file("dec").unwrap(),
    ///     )
    ///     .unwrap();
    ///
    /// assert_eq!(tmp_dir.read_file("dec").unwrap(), my_super_secret_message);
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

fn hash_key<K>(key: K) -> Vec<u8>
where
    K: AsRef<[u8]>,
{
    digest::digest(&digest::SHA256, key.as_ref())
        .as_ref()
        .to_owned()
}

fn random_iv() -> Vec<u8> {
    let mut iv = [0; IV_LEN];
    OsRng.fill_bytes(&mut iv);
    iv.to_vec()
}
