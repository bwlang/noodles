use std::io::{self, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use noodles_csi::binning_index::index::reference_sequence::{Bin, Metadata};

use crate::io::writer::num::write_u32_le;

pub(super) fn write_metadata<W>(writer: &mut W, metadata: &Metadata) -> io::Result<()>
where
    W: Write,
{
    use crate::index::DEPTH;

    const METADATA_ID: usize = Bin::metadata_id(DEPTH);
    const METADATA_CHUNK_COUNT: usize = 2;

    let bin_id =
        u32::try_from(METADATA_ID).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    write_u32_le(writer, bin_id)?;

    let n_chunk = u32::try_from(METADATA_CHUNK_COUNT)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    write_u32_le(writer, n_chunk)?;

    let ref_beg = u64::from(metadata.start_position());
    writer.write_u64::<LittleEndian>(ref_beg)?;

    let ref_end = u64::from(metadata.end_position());
    writer.write_u64::<LittleEndian>(ref_end)?;

    let n_mapped = metadata.mapped_record_count();
    writer.write_u64::<LittleEndian>(n_mapped)?;

    let n_unmapped = metadata.unmapped_record_count();
    writer.write_u64::<LittleEndian>(n_unmapped)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use noodles_bgzf as bgzf;

    use super::*;

    #[test]
    fn test_write_metadata() -> io::Result<()> {
        let metadata = Metadata::new(
            bgzf::VirtualPosition::from(610),
            bgzf::VirtualPosition::from(1597),
            55,
            0,
        );

        let mut buf = Vec::new();
        write_metadata(&mut buf, &metadata)?;

        let expected = [
            0x4a, 0x92, 0x00, 0x00, // bin = 37450
            0x02, 0x00, 0x00, 0x00, // chunks = 2
            0x62, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ref_beg = 610
            0x3d, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ref_end = 1597
            0x37, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // n_mapped = 55
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // n_unmapped = 0
        ];

        assert_eq!(buf, expected);

        Ok(())
    }
}
