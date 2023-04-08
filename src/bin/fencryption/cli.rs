use std::path::PathBuf;

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fencryption", version)]
/// A program to encrypt/decrypt text, files and directories
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Enable debug log
    #[clap(short = 'D', long, required = false, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encrypt text or files and directories
    Encrypt {
        #[clap(subcommand)]
        command: EncryptCommands,
    },
    /// Decrypt text or files and directories
    Decrypt {
        #[clap(subcommand)]
        command: DecryptCommands,
    },
}

#[derive(Subcommand)]
pub enum EncryptCommands {
    File {
        /// Paths of files and directories to encrypt
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Set output path (only supported when one input path
        /// is provided)
        #[arg(short, long)]
        output_path: Option<PathBuf>,

        /// Overwrite output files and directories
        #[clap(short = 'O', long)]
        overwrite: bool,

        /// Delete original files and directories after encrypting
        #[clap(short = 'd', long)]
        delete_original: bool,

        #[clap(from_global)]
        debug: bool,
    },
    Text {
        /// Text to encrypt
        #[arg(required = true)]
        text: String,
    },
}

#[derive(Subcommand)]
pub enum DecryptCommands {
    File {
        /// Paths of files and directories to encrypt
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Set output path (only supported when one input path
        /// is provided)
        #[arg(short, long)]
        output_path: Option<PathBuf>,

        /// Overwrite output files and directories
        #[clap(short = 'O', long)]
        overwrite: bool,

        /// Delete original files and directories after decrypting
        #[clap(short = 'd', long)]
        delete_original: bool,

        #[clap(from_global)]
        debug: bool,
    },
    Text {
        /// Text to decrypt (in base64)
        #[arg(required = true)]
        encrypted: String,
    },
}
