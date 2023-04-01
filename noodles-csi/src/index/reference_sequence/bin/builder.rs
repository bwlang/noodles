use noodles_bgzf as bgzf;

use super::{Bin, Chunk};

/// A CSI index reference sequence bin builder.
#[derive(Debug)]
pub struct Builder {
    id: usize,
    pub(crate) loffset: bgzf::VirtualPosition,
    chunks: Vec<Chunk>,
}

impl Builder {
    /// Sets the ID.
    pub fn set_id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }

    /// Adds or merges a chunk.
    pub fn add_chunk(&mut self, chunk: Chunk) {
        if chunk.start() < self.loffset {
            self.loffset = chunk.start();
        }

        if let Some(last_chunk) = self.chunks.last_mut() {
            if chunk.start() <= last_chunk.end() {
                *last_chunk = Chunk::new(last_chunk.start(), chunk.end());
                return;
            }
        }

        self.chunks.push(chunk);
    }

    /// Builds a bin.
    pub fn build(self) -> Bin {
        Bin {
            id: self.id,
            loffset: self.loffset,
            chunks: self.chunks,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            id: 0,
            loffset: bgzf::VirtualPosition::MAX,
            chunks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let builder = Builder::default();
        assert_eq!(builder.id, 0);
        assert_eq!(builder.loffset, bgzf::VirtualPosition::MAX);
        assert!(builder.chunks.is_empty());
    }

    #[test]
    fn test_set_id() {
        let builder = Builder::default().set_id(8);
        assert_eq!(builder.id, 8);
    }

    #[test]
    fn test_add_chunk() {
        let mut builder = Builder::default();

        assert!(builder.chunks.is_empty());

        builder.add_chunk(Chunk::new(
            bgzf::VirtualPosition::from(5),
            bgzf::VirtualPosition::from(13),
        ));

        assert_eq!(
            builder.chunks,
            [Chunk::new(
                bgzf::VirtualPosition::from(5),
                bgzf::VirtualPosition::from(13)
            )]
        );

        builder.add_chunk(Chunk::new(
            bgzf::VirtualPosition::from(8),
            bgzf::VirtualPosition::from(21),
        ));

        assert_eq!(
            builder.chunks,
            [Chunk::new(
                bgzf::VirtualPosition::from(5),
                bgzf::VirtualPosition::from(21)
            )]
        );

        builder.add_chunk(Chunk::new(
            bgzf::VirtualPosition::from(34),
            bgzf::VirtualPosition::from(55),
        ));

        assert_eq!(
            builder.chunks,
            [
                Chunk::new(
                    bgzf::VirtualPosition::from(5),
                    bgzf::VirtualPosition::from(21)
                ),
                Chunk::new(
                    bgzf::VirtualPosition::from(34),
                    bgzf::VirtualPosition::from(55)
                )
            ]
        );
    }

    #[test]
    fn test_build() {
        let mut builder = Builder::default().set_id(8);

        builder.add_chunk(Chunk::new(
            bgzf::VirtualPosition::from(5),
            bgzf::VirtualPosition::from(13),
        ));

        let actual = builder.build();

        let expected = Bin {
            id: 8,
            loffset: bgzf::VirtualPosition::from(5),
            chunks: vec![Chunk::new(
                bgzf::VirtualPosition::from(5),
                bgzf::VirtualPosition::from(13),
            )],
        };

        assert_eq!(actual, expected);
    }
}
