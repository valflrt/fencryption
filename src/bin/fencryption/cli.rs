use std::path::PathBuf;

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fencryption", version)]
/// A program to encrypt/decrypt text and files
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Enable debug log
    #[clap(short = 'D', long, required = false, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encrypt text or files
    Encrypt {
        #[clap(subcommand)]
        command: EncryptCommands,
    },
    /// Decrypt text or files
    Decrypt {
        #[clap(subcommand)]
        command: DecryptCommands,
    },
}

#[derive(Subcommand)]
pub enum EncryptCommands {
    /// Encrypt files and directories
    File {
        /// Paths of the files to encrypt
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Set output path (only supported when one input path
        /// is provided)
        #[arg(short, long)]
        output_path: Option<PathBuf>,

        /// Overwrite output
        #[clap(short = 'O', long)]
        overwrite: bool,

        /// Delete original files after encrypting
        #[clap(short = 'd', long)]
        delete_original: bool,

        #[clap(from_global)]
        debug: bool,
    },
    /// Encrypt text
    Text {
        /// Text to encrypt
        #[arg(required = true)]
        text: String,
    },
}

#[derive(Subcommand)]
pub enum DecryptCommands {
    /// Decrypt files and directories
    File {
        /// Paths of files to encrypt
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Set output path (only supported when one input path
        /// is provided)
        #[arg(short, long)]
        output_path: Option<PathBuf>,

        /// Overwrite output
        #[clap(short = 'O', long)]
        overwrite: bool,

        /// Delete encrypted files after decrypting
        #[clap(short = 'd', long)]
        delete_original: bool,

        #[clap(from_global)]
        debug: bool,
    },
    /// Decrypt text
    Text {
        /// Text to decrypt (base64 encoded)
        #[arg(required = true)]
        encrypted: String,
    },
}
