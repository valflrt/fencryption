use std::{
    fs::File,
    io::{self, Read, Write},
};

use crate::constants::DEFAULT_CHUNK_LEN;

fn move_chunk(
    buf: &mut [u8; DEFAULT_CHUNK_LEN],
    from: &mut File,
    to: &mut File,
) -> io::Result<usize> {
    let read_len = from.read(buf)?;
    to.write(&buf[..read_len])?;
    Ok(read_len)
}

/// Transfers data from a file to another.
pub fn stream(from: &mut File, to: &mut File) -> io::Result<()> {
    let mut buffer = [0u8; DEFAULT_CHUNK_LEN];
    loop {
        let read_len = move_chunk(&mut buffer, from, to)?;
        if read_len != DEFAULT_CHUNK_LEN {
            break;
        }
    }
    Ok(())
}

/// Chain struct that enables to chain two implementors of
/// the Read trait.
pub struct Chain<R1: Read, R2: Read>(R1, R2, bool);

impl<R1: Read, R2: Read> Chain<R1, R2> {
    /// Creates a Chain that pulls bytes from the first
    /// Reader until it reaches its EOF. When it is reached,
    /// pulls bytes from the second Reader.
    pub fn new(first: R1, second: R2) -> Self {
        Chain(first, second, false)
    }
}

impl<R1: Read, R2: Read> Read for Chain<R1, R2> {
    /// Pull some bytes from this source into the specified
    /// buffer, returning how many bytes were read. The function
    /// first pulls bytes from the prepend buffer.
    ///
    /// Example:
    ///
    /// ```
    /// use fencryption_lib::io::Chain;
    /// use std::io::Read;
    ///
    /// let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    /// //
    /// let mut source = Chain::new([255u8; 53].as_ref(), lorem.as_bytes());
    ///
    /// let mut buf = vec![0u8; 16];
    /// loop {
    ///     let read_len = source.read(&mut buf).unwrap();
    ///     println!("{:x?} {}", &buf[..read_len], read_len);
    ///     if read_len != buf.len() {
    ///         break;
    ///     }
    /// }
    /// ```
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        if self.2 {
            Ok(self.1.read(buf)?)
        } else {
            let buf_len = buf.len();
            match self.0.read(&mut buf)? {
                n if n < buf_len => {
                    let mut from_second = vec![0u8; buf_len - n];
                    let n2 = self.1.read(&mut from_second)?;
                    buf.write(&[&buf[..n], &from_second].concat())?;
                    self.2 = true;
                    Ok(n + n2)
                }
                n => Ok(n),
            }
        }
    }
}
