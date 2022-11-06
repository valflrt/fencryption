use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

pub struct TmpDir(PathBuf);

impl TmpDir {
    pub fn new() -> Result<TmpDir, io::Error> {
        let path = env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        if let Err(e) = fs::create_dir(&path) {
            return Err(e);
        };
        Ok(TmpDir(path))
    }

    pub fn path(&self) -> PathBuf {
        self.0.to_owned()
    }

    /// Generates a new unique path in the temporary directory.
    pub fn gen_path(&self) -> PathBuf {
        self.0.join(uuid::Uuid::new_v4().to_string())
    }

    /// Writes to a file (or create it if it doesn't exist) in the TmpDir, file_path must be
    /// relative. See [std::fs::write].
    pub fn write_file<P, C>(&self, file_path: P, contents: C) -> Result<(), std::io::Error>
    where
        P: AsRef<Path>,
        C: AsRef<[u8]>,
    {
        assert!(file_path.as_ref().is_relative());
        fs::write(self.path().join(file_path), contents)
    }

    /// Reads a file from the TmpDir, file_path must be
    /// relative. See [std::fs::read].
    pub fn read_file<P>(&self, file_path: P) -> Result<Vec<u8>, std::io::Error>
    where
        P: AsRef<Path>,
    {
        assert!(file_path.as_ref().is_relative());
        fs::read(self.path().join(file_path))
    }

    /// Creates a file in the TmpDir, file_path must be
    /// relative. See [std::fs::File::create].
    pub fn create_file<P>(&self, file_path: P) -> Result<File, std::io::Error>
    where
        P: AsRef<Path>,
    {
        assert!(file_path.as_ref().is_relative());
        File::create(self.path().join(file_path))
    }

    /// Opens a file from the TmpDir, file_path must be
    /// relative. See [std::fs::File::open].
    pub fn open_file<P>(&self, file_path: P) -> Result<File, std::io::Error>
    where
        P: AsRef<Path>,
    {
        assert!(file_path.as_ref().is_relative());
        File::open(self.path().join(file_path))
    }
}

/// Impl Drop trait so when the TmpDir is dropped, the directory
/// is deleted.
impl Drop for TmpDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}
