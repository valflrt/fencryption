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
