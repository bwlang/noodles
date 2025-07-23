use std::io::{self, Read};

use crate::{
    codecs::rans_4x8::Order,
    io::reader::num::{read_u8, read_u32_le},
};

pub(super) fn read_header<R>(reader: &mut R) -> io::Result<(Order, usize, usize)>
where
    R: Read,
{
    let order = read_order(reader)?;
    let compressed_size = read_size(reader)?;
    let uncompressed_size = read_size(reader)?;
    Ok((order, compressed_size, uncompressed_size))
}

fn read_order<R>(reader: &mut R) -> io::Result<Order>
where
    R: Read,
{
    match read_u8(reader)? {
        0 => Ok(Order::Zero),
        1 => Ok(Order::One),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid order")),
    }
}

fn read_size<R>(reader: &mut R) -> io::Result<usize>
where
    R: Read,
{
    read_u32_le(reader)
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_header() -> io::Result<()> {
        let src = [
            0x00, // order 0
            0x08, 0x00, 0x00, 0x00, // compressed size = 8
            0x0d, 0x00, 0x00, 0x00, // uncompressed size = 13
        ];

        assert_eq!(read_header(&mut &src[..])?, (Order::Zero, 8, 13));

        Ok(())
    }

    #[test]
    fn test_read_order() -> io::Result<()> {
        assert_eq!(read_order(&mut &[0x00][..])?, Order::Zero);
        assert_eq!(read_order(&mut &[0x01][..])?, Order::One);

        assert!(matches!(
            read_order(&mut &[0x02][..]),
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));

        Ok(())
    }
}
