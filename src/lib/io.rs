//! IO utilities.

use std::io::{self, Read, Write};

/// Default buffer length for io (4kb).
pub const DEFAULT_BUF_LEN: usize = 4000;

/// Transfer data from a reader to a writer.
pub fn stream(from: &mut impl Read, to: &mut impl Write) -> io::Result<()> {
    let mut buffer = [0u8; DEFAULT_BUF_LEN];
    loop {
        let read_len = from.read(&mut buffer)?;
        to.write_all(&buffer[..read_len])?;
        if read_len != DEFAULT_BUF_LEN {
            break;
        }
    }
    Ok(())
}

/// Adapter to chain two readers together.
///
/// It might seem the exact same as [`std::io::Chain`] but it
/// is not. The difference is that when it reaches the end of
/// the first reader, it fills the rest of the buffer with
/// the first bytes from the second reader, what [`std::io::Chain`]
/// doesn't do.
pub struct Chain<R1: Read, R2: Read> {
    first: Option<R1>,
    second: R2,
}

impl<R1: Read, R2: Read> Chain<R1, R2> {
    /// Create a Chain from the two given readers.
    pub fn new(first: R1, second: R2) -> Self {
        Chain {
            first: Some(first),
            second,
        }
    }
}

impl<R1: Read, R2: Read> Read for Chain<R1, R2> {
    /// Pull some bytes from this source into the specified
    /// buffer, returning how many bytes were read.
    ///
    /// Example:
    ///
    /// ```
    /// use std::io::Read;
    /// use fencryption_lib::io::Chain;
    ///
    /// let text = "Never gonna give you up ! Never gonna let you down !";
    /// let mut source = Chain::new([255u8; 41].as_ref(), text.as_bytes());
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
    ///
    /// Output:
    ///
    /// ```sh
    /// [ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff] 16
    /// [ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff, ff] 16
    /// [ff, ff, ff, ff, ff, ff, ff, ff, ff, 4e, 65, 76, 65, 72, 20, 67] 16
    /// [6f, 6e, 6e, 61, 20, 67, 69, 76, 65, 20, 79, 6f, 75, 20, 75, 70] 16
    /// [20, 21, 20, 4e, 65, 76, 65, 72, 20, 67, 6f, 6e, 6e, 61, 20, 6c] 16
    /// [65, 74, 20, 79, 6f, 75, 20, 64, 6f, 77, 6e, 20, 21] 13
    /// ```
    fn read(&mut self, out_buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.first {
            Some(first) => {
                let buf_len = out_buf.len();
                match first.read(out_buf)? {
                    n if n < buf_len => {
                        self.first = None;
                        Ok(n + self.second.read(&mut out_buf[n..])?)
                    }
                    n => Ok(n),
                }
            }
            None => Ok(self.second.read(out_buf)?),
        }
    }
}
