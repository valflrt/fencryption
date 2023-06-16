//! Handle temporary files and directories.

use std::{
    env,
    fs::{self, File, OpenOptions, ReadDir},
    io,
    path::{Path, PathBuf},
};

/// Handle a temporary directory.
///
/// The `path` parameter (present in some methods) must be
/// relative because it will be joined to the temporary
/// directory path.
///
/// When this struct is dropped, the temporary directory
/// itself is automatically deleted.
pub struct TmpDir(PathBuf);

impl TmpDir {
    /// Create a new TmpDir instance.
    pub fn new() -> Result<Self, io::Error> {
        let path = env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        fs::create_dir(&path)?;
        Ok(TmpDir(path))
    }

    /// Path of the temporary directory.
    pub fn path(&self) -> PathBuf {
        self.0.clone()
    }

    /// Generate a new unique path in the temporary directory.
    pub fn unique_path(&self) -> PathBuf {
        self.0.join(uuid::Uuid::new_v4().to_string())
    }

    /// Write to a file (or create it if it doesn't exist)
    /// in the temporary directory. Akin to [`fs::write`].
    pub fn write_file<P, C>(&self, path: P, contents: C) -> io::Result<()>
    where
        P: AsRef<Path>,
        C: AsRef<[u8]>,
    {
        fs::write(self.0.join(path), contents)
    }

    /// Read a file in the temporary directory. Akin to
    /// [`fs::read`].
    pub fn read_file<P>(&self, path: P) -> io::Result<Vec<u8>>
    where
        P: AsRef<Path>,
    {
        fs::read(self.0.join(path))
    }

    /// Create a directory inside the temporary directory.
    /// Akin to [`fs::create_dir`].
    pub fn create_dir<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::create_dir(self.0.join(path))
    }

    /// Create a directory and all of its parent components
    /// if they are missing. Akin to [`fs::create_dir_all`].
    pub fn create_dir_all<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::create_dir_all(self.0.join(path))
    }

    /// Create a file in the temporary directory and open it
    /// in write-only mode. Akin to [`File::create`].
    pub fn create_file<P>(&self, path: P) -> io::Result<File>
    where
        P: AsRef<Path>,
    {
        File::create(self.0.join(path))
    }

    /// Open a file in the temporary directory in read-only
    /// mode. Akin to [`File::open`].
    pub fn open_readable<P>(&self, path: P) -> io::Result<File>
    where
        P: AsRef<Path>,
    {
        File::open(self.0.join(path))
    }

    /// Open a file in the temporary directory in write-only
    /// mode.
    pub fn open_writable<P>(&self, path: P) -> io::Result<File>
    where
        P: AsRef<Path>,
    {
        OpenOptions::new().write(true).open(self.0.join(path))
    }

    /// Open a file in the temporary directory using the
    /// provided OpenOptions. Akin to [`fs::OpenOptions::open`].
    pub fn open_with_opts<P>(&self, opts: &mut OpenOptions, path: P) -> io::Result<File>
    where
        P: AsRef<Path>,
    {
        opts.open(self.0.join(path))
    }

    /// Get metadata for the given path. Akin to [`fs::metadata`].
    pub fn metadata<P>(&self, path: P) -> io::Result<fs::Metadata>
    where
        P: AsRef<Path>,
    {
        self.0.join(path.as_ref()).metadata()
    }

    /// Check if a path exists in the current directory. Akin
    /// to [`Path::exists`].
    pub fn exists<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.0.join(path.as_ref()).exists()
    }

    /// Read temporary directory. Akin to [`fs::read_dir`].
    pub fn read_dir<P>(&self, path: P) -> io::Result<ReadDir>
    where
        P: AsRef<Path>,
    {
        fs::read_dir(self.0.join(path))
    }

    /// Remove a file in the current directory. Akin to
    /// [`fs::remove_file`].
    pub fn remove_file<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::remove_file(self.0.join(path))
    }

    /// Remove a directory in the current directory. Akin to
    /// [`fs::remove_dir`].
    pub fn remove_dir<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::remove_dir(self.0.join(path))
    }

    /// Remove a directory and all its contents in the current
    /// directory. Akin to [`fs::remove_dir_all`].
    pub fn remove_dir_all<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::remove_dir_all(self.0.join(path))
    }
}

impl Drop for TmpDir {
    /// When TmpDir is dropped, the temporary directory
    /// itself is deleted.
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}

/// Handle a temporary file.
///
/// When this struct is dropped, the temporary file itself is
/// automatically deleted.
pub struct TmpFile(PathBuf);

impl TmpFile {
    /// Create a new TmpFile instance.
    pub fn new() -> Result<Self, io::Error> {
        let path = env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        fs::write(&path, [])?;
        Ok(TmpFile(path))
    }

    /// Path of the temporary file.
    pub fn path(&self) -> PathBuf {
        self.0.clone()
    }

    /// Write to the temporary file. Akin to [`fs::write`].
    pub fn write<C>(&self, contents: C) -> io::Result<()>
    where
        C: AsRef<[u8]>,
    {
        fs::write(&self.0, contents)
    }

    /// Read the temporary file. Akin to [`fs::read`].
    pub fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.0)
    }

    /// Open the temporary file in read-only mode. Akin to
    /// [`File::open`].
    pub fn open_readable(&self) -> io::Result<File> {
        File::open(&self.0)
    }

    /// Open the temporary file in write-only mode.
    pub fn open_writable(&self) -> io::Result<File> {
        OpenOptions::new().write(true).open(&self.0)
    }

    /// Open the temporary file using the provided OpenOptions.
    /// Akin to [`fs::OpenOptions::open`].
    pub fn open_with_opts(&self, opts: &mut OpenOptions) -> io::Result<File> {
        opts.open(&self.0)
    }
}

impl Drop for TmpFile {
    /// When TmpFile is dropped, the temporary file itself is
    /// deleted.
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}
