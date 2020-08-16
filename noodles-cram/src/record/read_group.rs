use std::ops::Deref;

// § 8.4 Compression header block (2020-06-22): "Special value '-1' stands for no group."
const NULL: i32 = -1;
const MIN: i32 = 0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReadGroup(Option<i32>);

impl Deref for ReadGroup {
    type Target = Option<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for ReadGroup {
    fn from(n: i32) -> Self {
        if n < MIN {
            Self(None)
        } else {
            Self(Some(n))
        }
    }
}

impl From<ReadGroup> for i32 {
    fn from(read_group: ReadGroup) -> Self {
        match *read_group {
            Some(id) => id,
            None => NULL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i32_for_read_group() {
        assert_eq!(*ReadGroup::from(-1), None);
        assert_eq!(*ReadGroup::from(13), Some(13));
    }

    #[test]
    fn test_from_read_group_for_i32() {
        assert_eq!(i32::from(ReadGroup(None)), -1);
        assert_eq!(i32::from(ReadGroup(Some(13))), 13);
    }
}
