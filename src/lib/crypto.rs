//! Crypto utility.

use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

use crate::io::DEFAULT_BUF_LEN;

/// Default initialization vector length (12b).
pub const IV_LEN: usize = 96 / 8;
/// Default authentication tag length (16b).
pub const TAG_LEN: usize = 128 / 8;
/// Encrypted assets length: iv (12b) + auth tag (16b).
pub const ENC_ASSETS_LEN: usize = IV_LEN + TAG_LEN;

/// Plain chunk length: plaintext (128kb).
pub const PLAIN_CHUNK_LEN: usize = DEFAULT_BUF_LEN;

/// Encrypted chunk length: iv (12b) + ciphertext (128kb) + auth tag (16b).
pub const ENC_CHUNK_LEN: usize = PLAIN_CHUNK_LEN + ENC_ASSETS_LEN;

/// Enum of the different possible crypto errors.
#[derive(Debug)]
pub enum ErrorKind {
    AesError(aes_gcm::Error),
    Io(io::Error),
}

/// A struct for encrypting/decrypting bytes or io streams.
#[derive(Clone)]
pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Create a new [`Crypto`] instance, the given key will be
    /// used for every operation performed.
    pub fn new<K>(key: K) -> Result<Crypto, ErrorKind>
    where
        K: AsRef<[u8]>,
    {
        let key = hash_key(key.as_ref());
        Ok(Crypto {
            cipher: Aes256Gcm::new_from_slice(&key).unwrap(),
        })
    }

    /// Encrypt bytes with initialisation vector.
    pub fn encrypt_with_iv(&self, buf: &[u8], iv: &[u8]) -> io::Result<Vec<u8>> {
        self.cipher
            .encrypt(Nonce::from_slice(iv), buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to encrypt"))
    }

    /// Decrypt bytes with initialisation vector.
    pub fn decrypt_with_iv(&self, ciphertext: &[u8], iv: &[u8]) -> io::Result<Vec<u8>> {
        self.cipher
            .decrypt(Nonce::from_slice(iv), ciphertext)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to decrypt"))
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
    pub fn encrypt(&self, plain: impl AsRef<[u8]>) -> io::Result<Vec<u8>> {
        let iv = random_iv();
        Ok([&iv, self.encrypt_with_iv(plain.as_ref(), &iv)?.as_slice()].concat())
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
    pub fn decrypt(&self, enc: impl AsRef<[u8]>) -> io::Result<Vec<u8>> {
        let (iv, ciphertext) = enc.as_ref().split_at(IV_LEN);
        self.decrypt_with_iv(ciphertext, iv)
    }

    /// Encrypt data from a reader and write it in a writer.
    /// When working with small pieces of data, use
    /// [`Crypto::encrypt`].
    ///
    /// Example:
    ///
    /// (See [`TmpDir`][crate::tmp::TmpDir])
    ///
    /// ```
    /// use fencryption_lib::crypto::Crypto;
    /// use fencryption_lib::tmp::TmpDir;
    ///
    /// let my_super_key = b"this_is_super_secure";
    /// let my_super_secret_message = b"hello :)";
    ///
    /// let tmp_dir = TmpDir::new().unwrap();
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// tmp_dir.write_file("plain", my_super_secret_message).unwrap();
    ///
    /// crypto
    ///     .encrypt_io(
    ///         &mut tmp_dir.open_readable("plain").unwrap(),
    ///         &mut tmp_dir.create_file("enc").unwrap(),
    ///     )
    ///     .unwrap();
    /// ```
    pub fn encrypt_io(&self, source: &mut impl Read, dest: &mut impl Write) -> io::Result<()> {
        let mut buffer = [0u8; PLAIN_CHUNK_LEN];

        loop {
            let read_len = source.read(&mut buffer)?;
            dest.write_all(&self.encrypt(&buffer[..read_len])?)?;
            // Stops when the loop reached the end of the file
            if read_len != PLAIN_CHUNK_LEN {
                break;
            }
        }

        Ok(())
    }

    /// Decrypt data from a reader and write it in a writer.
    /// When working with small pieces of data, use
    /// [`Crypto::decrypt`].
    ///
    /// Example:
    ///
    /// (See [`TmpDir`][crate::tmp::TmpDir])
    ///
    /// ```
    /// use fencryption_lib::crypto::Crypto;
    /// use fencryption_lib::tmp::TmpDir;
    ///
    /// let my_super_key = b"this_is_super_secure";
    /// let my_super_secret_message = b"hello :)";
    ///
    /// let tmp_dir = TmpDir::new().unwrap();
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// tmp_dir.write_file("plain", my_super_secret_message).unwrap();
    ///
    /// crypto
    ///     .encrypt_io(
    ///         &mut tmp_dir.open_readable("plain").unwrap(),
    ///         &mut tmp_dir.create_file("enc").unwrap(),
    ///     )
    ///     .unwrap();
    /// crypto
    ///     .decrypt_io(
    ///         &mut tmp_dir.open_readable("enc").unwrap(),
    ///         &mut tmp_dir.create_file("dec").unwrap(),
    ///     )
    ///     .unwrap();
    ///
    /// assert_eq!(tmp_dir.read_file("dec").unwrap(), my_super_secret_message[..]);
    /// ```
    pub fn decrypt_io(&self, source: &mut impl Read, dest: &mut impl Write) -> io::Result<()> {
        let mut buffer = [0u8; ENC_CHUNK_LEN];

        loop {
            let read_len = source.read(&mut buffer)?;
            dest.write_all(&self.decrypt(&buffer[..read_len])?)?;
            // Stops when the loop reached the end of the file.
            if read_len != ENC_CHUNK_LEN {
                break;
            }
        }

        Ok(())
    }
}

pub struct EncryptCipher<R: Read> {
    source: R,
    crypto: Crypto,
    buf: [u8; ENC_CHUNK_LEN],
    start: usize,
    end: usize,
}

impl<R: Read> EncryptCipher<R> {
    pub fn new<K>(key: K, source: R) -> Self
    where
        K: AsRef<[u8]>,
    {
        EncryptCipher {
            source,
            crypto: Crypto::new(key),
            buf: [0u8; ENC_CHUNK_LEN],
            start: ENC_CHUNK_LEN,
            end: ENC_CHUNK_LEN,
        }
    }
}

impl<R: Read> Read for EncryptCipher<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;
        let buf_len = buf.len();

        loop {
            if self.start >= self.end {
                let read_len = self.source.read(&mut self.buf[..PLAIN_CHUNK_LEN])?;
                if read_len == 0 {
                    break;
                }

                let enc = self.crypto.encrypt(&self.buf[..read_len])?;

                self.start = 0;
                self.end = enc.len();

                self.buf.as_mut().write(&enc)?;
            }

            if self.start + buf_len < self.end {
                buf.copy_from_slice(&self.buf[self.start..self.start + buf_len]);
                total += buf_len;
                self.start += buf_len;
                break;
            } else {
                let remaining_len = self.end - self.start;
                buf[..remaining_len]
                    .copy_from_slice(&self.buf[self.start..self.start + remaining_len]);
                total += remaining_len;
                self.start = self.end;
            }
        }

        Ok(total)
    }
}

pub struct DecryptCipher<R: Read> {
    source: R,
    crypto: Crypto,
    buf: [u8; ENC_CHUNK_LEN],
    start: usize,
    end: usize,
}

impl<R: Read> DecryptCipher<R> {
    pub fn new<K>(key: K, source: R) -> Self
    where
        K: AsRef<[u8]>,
    {
        DecryptCipher {
            source,
            crypto: Crypto::new(key),
            buf: [0u8; ENC_CHUNK_LEN],
            start: ENC_CHUNK_LEN,
            end: ENC_CHUNK_LEN,
        }
    }
}

impl<R: Read> Read for DecryptCipher<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;
        let buf_len = buf.len();

        loop {
            if self.start >= self.end {
                let read_len = self.source.read(&mut self.buf[..ENC_CHUNK_LEN])?;
                if read_len == 0 {
                    break;
                }

                let dec = self.crypto.decrypt(&self.buf[..read_len])?;

                self.start = 0;
                self.end = dec.len();

                self.buf.as_mut().write(&dec)?;
            }

            if self.start + buf_len < self.end {
                buf.copy_from_slice(&self.buf[self.start..self.start + buf_len]);
                total += buf_len;
                self.start += buf_len;
                break;
            } else {
                let remaining_len = self.end - self.start;
                buf[..remaining_len]
                    .copy_from_slice(&self.buf[self.start..self.start + remaining_len]);
                total += remaining_len;
                self.start = self.end;
            }
        }

        Ok(total)
    }
}

fn hash_key<K>(key: K) -> Vec<u8>
where
    K: AsRef<[u8]>,
{
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(key.as_ref());
    hasher.finalize().to_vec()
}

fn random_iv() -> [u8; IV_LEN] {
    let mut iv = [0; IV_LEN];
    OsRng.fill_bytes(&mut iv);
    iv
}
