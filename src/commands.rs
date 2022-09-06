use clap::Subcommand;

pub mod decrypt;
pub mod decrypt_stream;
pub mod encrypt;
pub mod encrypt_stream;

#[derive(Subcommand)]
pub enum Commands {
    Encrypt(encrypt::Command),
    Decrypt(decrypt::Command),
    EncryptStream(encrypt_stream::Command),
    DecryptStream(decrypt_stream::Command),
}
