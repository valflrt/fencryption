use std::{
    env,
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

/// TmpDir is a struct to manipulate a temporary directory.
///
/// The "path" argument (in some methods) must be relative
/// because it will be joined to the temporary directory path.
///
/// When this struct is dropped, the temporary directory
/// itself is automatically deleted.
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

    /// Writes to a file (or create it if it doesn't exist)
    /// in the TmpDir. See [std::fs::write].
    pub fn write_file<P, C>(&self, path: P, contents: C) -> io::Result<()>
    where
        P: AsRef<Path>,
        C: AsRef<[u8]>,
    {
        fs::write(self.path().join(path), contents)
    }

    /// Reads a file from the TmpDir. See [std::fs::read].
    pub fn read_file<P>(&self, path: P) -> io::Result<Vec<u8>>
    where
        P: AsRef<Path>,
    {
        fs::read(self.path().join(path))
    }

    /// Creates a directory. See [std::fs::create_dir].
    pub fn create_dir<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::create_dir(self.path().join(path))
    }

    /// Creates a directory and all of its parent if they are
    /// missing. See [std::fs::create_dir_all].
    pub fn create_dir_all<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::create_dir_all(self.path().join(path))
    }

    /// Creates a file in the TmpDir. See [std::fs::File::create].
    pub fn create_file<P>(&self, path: P) -> io::Result<File>
    where
        P: AsRef<Path>,
    {
        File::create(self.path().join(path))
    }

    /// Opens a file from the TmpDir. See [std::fs::File::open].
    pub fn open_file<P>(&self, path: P) -> io::Result<File>
    where
        P: AsRef<Path>,
    {
        File::open(self.path().join(path))
    }

    /// Opens a file from the TmpDir using the provided
    /// OpenOptions. See [std::fs::OpenOptions::open].
    pub fn open_file_with_opts<P>(&self, opts: OpenOptions, path: P) -> io::Result<File>
    where
        P: AsRef<Path>,
    {
        opts.open(self.path().join(path))
    }
}

/// Impl Drop trait so when the TmpDir is dropped, the directory
/// is deleted.
impl Drop for TmpDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}
