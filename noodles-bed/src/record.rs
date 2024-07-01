pub(crate) mod fields;
mod other_fields;

use std::{fmt, io};

use bstr::ByteSlice;
use noodles_core::Position;

use self::fields::Fields;
pub use self::other_fields::OtherFields;

/// A BED record.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Record(pub(crate) Fields);

impl Record {
    /// Returns the reference sequence name.
    pub fn reference_sequence_name(&self) -> &[u8] {
        self.0.reference_sequence_name()
    }

    /// Returns the feature start.
    pub fn feature_start(&self) -> io::Result<Position> {
        self.0.feature_start()
    }

    /// Returns the feature end.
    pub fn feature_end(&self) -> io::Result<Position> {
        self.0.feature_end()
    }

    /// Returns the name.
    pub fn name(&self) -> Option<&[u8]> {
        self.0.name()
    }

    /// Returns the score.
    pub fn score(&self) -> Option<&[u8]> {
        self.0.score()
    }

    /// Returns the strand.
    pub fn strand(&self) -> Option<&[u8]> {
        self.0.strand()
    }

    /// Returns the other fields.
    pub fn other_fields(&self) -> OtherFields<'_> {
        OtherFields::new(&self.0)
    }

    /// Returns the number of fields.
    ///
    /// This is guaranteed to be >= 3.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        const MIN_FIELD_COUNT: usize = 3;
        MIN_FIELD_COUNT + self.other_fields().len()
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Record")
            .field(
                "reference_sequence_name",
                &self.reference_sequence_name().as_bstr(),
            )
            .field("feature_start", &self.feature_start())
            .field("feature_end", &self.feature_end())
            .finish_non_exhaustive()
    }
}
