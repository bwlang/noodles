use std::{
    io::{self, Read},
    mem,
};

pub(crate) fn read_u32_le<R>(reader: &mut R) -> io::Result<u32>
where
    R: Read,
{
    let mut buf = [0; mem::size_of::<u32>()];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}
