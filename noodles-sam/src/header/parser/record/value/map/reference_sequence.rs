use std::{error, fmt, num::NonZeroUsize};

use super::field::{consume_delimiter, consume_separator, parse_tag, parse_value};
use crate::header::{
    parser::Context,
    record::value::{
        map::{
            self,
            reference_sequence::{
                alternative_locus, alternative_names, md5_checksum, molecule_topology, tag,
                AlternativeLocus, AlternativeNames, Md5Checksum, MoleculeTopology, Name, Tag,
            },
            OtherFields, ReferenceSequence,
        },
        Map,
    },
};

/// An error returned when a SAM header reference sequence record value fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidField(super::field::ParseError),

    MissingName,
    InvalidName(crate::record::reference_sequence_name::ParseError),
    MissingLength,
    InvalidLength,
    InvalidAlternativeLocus(alternative_locus::ParseError),
    InvalidAlternativeNames(alternative_names::ParseError),
    InvalidMd5Checksum(md5_checksum::ParseError),
    InvalidMoleculeTopology(molecule_topology::ParseError),

    DuplicateTag(Tag),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidField(e) => Some(e),
            Self::InvalidName(e) => Some(e),
            Self::InvalidAlternativeLocus(e) => Some(e),
            Self::InvalidAlternativeNames(e) => Some(e),
            Self::InvalidMd5Checksum(e) => Some(e),
            Self::InvalidMoleculeTopology(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidField(_) => write!(f, "invalid field"),
            Self::MissingName => write!(f, "missing name ({}) field", tag::NAME),
            Self::InvalidName(_) => write!(f, "invalid name"),
            Self::MissingLength => write!(f, "missing length ({}) field", tag::LENGTH),
            Self::InvalidLength => write!(f, "invalid length"),
            Self::InvalidAlternativeLocus(_) => write!(f, "invalid alternative locus"),
            Self::InvalidAlternativeNames(_) => write!(f, "invalid alternative names"),
            Self::InvalidMd5Checksum(_) => write!(f, "invalid MD5 checksum"),
            Self::InvalidMoleculeTopology(_) => write!(f, "invalid molecule topology"),
            Self::DuplicateTag(tag) => write!(f, "duplicate tag: {tag}"),
        }
    }
}

pub(crate) fn parse_reference_sequence(
    src: &mut &[u8],
    ctx: &Context,
) -> Result<(Name, Map<ReferenceSequence>), ParseError> {
    let mut name = None;
    let mut length = None;
    let mut alternative_locus = None;
    let mut alternative_names = None;
    let mut assembly_id = None;
    let mut description = None;
    let mut md5_checksum = None;
    let mut species = None;
    let mut molecule_topology = None;
    let mut uri = None;

    let mut other_fields = OtherFields::new();

    while !src.is_empty() {
        consume_delimiter(src).map_err(ParseError::InvalidField)?;
        let tag = parse_tag(src).map_err(ParseError::InvalidField)?;
        consume_separator(src).map_err(ParseError::InvalidField)?;

        match tag {
            tag::NAME => parse_name(src).and_then(|v| try_replace(&mut name, ctx, tag::NAME, v))?,
            tag::LENGTH => {
                parse_length(src).and_then(|v| try_replace(&mut length, ctx, tag::LENGTH, v))?
            }
            tag::ALTERNATIVE_LOCUS => parse_alternative_locus(src).and_then(|v| {
                try_replace(&mut alternative_locus, ctx, tag::ALTERNATIVE_LOCUS, v)
            })?,
            tag::ALTERNATIVE_NAMES => parse_alternative_names(src).and_then(|v| {
                try_replace(&mut alternative_names, ctx, tag::ALTERNATIVE_NAMES, v)
            })?,
            tag::ASSEMBLY_ID => parse_assembly_id(src)
                .and_then(|v| try_replace(&mut assembly_id, ctx, tag::ASSEMBLY_ID, v))?,
            tag::DESCRIPTION => parse_description(src)
                .and_then(|v| try_replace(&mut description, ctx, tag::DESCRIPTION, v))?,
            tag::MD5_CHECKSUM => parse_md5_checksum(src)
                .and_then(|v| try_replace(&mut md5_checksum, ctx, tag::MD5_CHECKSUM, v))?,
            tag::SPECIES => {
                parse_species(src).and_then(|v| try_replace(&mut species, ctx, tag::SPECIES, v))?
            }
            tag::MOLECULE_TOPOLOGY => parse_molecule_topology(src).and_then(|v| {
                try_replace(&mut molecule_topology, ctx, tag::MOLECULE_TOPOLOGY, v)
            })?,
            tag::URI => parse_uri(src).and_then(|v| try_replace(&mut uri, ctx, tag::URI, v))?,
            Tag::Other(t) => parse_value(src)
                .map_err(ParseError::InvalidField)
                .and_then(|value| try_insert(&mut other_fields, ctx, t, value))?,
        }
    }

    let name = name.ok_or(ParseError::MissingName)?;
    let length = length.ok_or(ParseError::MissingLength)?;

    Ok((
        name,
        Map {
            inner: ReferenceSequence {
                length,
                alternative_locus,
                alternative_names,
                assembly_id,
                description,
                md5_checksum,
                species,
                molecule_topology,
                uri,
            },
            other_fields,
        },
    ))
}

fn parse_name(src: &mut &[u8]) -> Result<Name, ParseError> {
    parse_value(src)
        .map_err(ParseError::InvalidField)
        .and_then(|s| s.parse().map_err(ParseError::InvalidName))
}

fn parse_length(src: &mut &[u8]) -> Result<NonZeroUsize, ParseError> {
    let (n, i) =
        lexical_core::parse_partial::<usize>(src).map_err(|_| ParseError::InvalidLength)?;

    *src = &src[i..];

    NonZeroUsize::try_from(n).map_err(|_| ParseError::InvalidLength)
}

fn parse_alternative_locus(src: &mut &[u8]) -> Result<AlternativeLocus, ParseError> {
    parse_value(src)
        .map_err(ParseError::InvalidField)
        .and_then(|s| s.parse().map_err(ParseError::InvalidAlternativeLocus))
}

fn parse_alternative_names(src: &mut &[u8]) -> Result<AlternativeNames, ParseError> {
    parse_value(src)
        .map_err(ParseError::InvalidField)
        .and_then(|s| s.parse().map_err(ParseError::InvalidAlternativeNames))
}

fn parse_assembly_id(src: &mut &[u8]) -> Result<String, ParseError> {
    parse_value(src)
        .map(String::from)
        .map_err(ParseError::InvalidField)
}

fn parse_description(src: &mut &[u8]) -> Result<String, ParseError> {
    parse_value(src)
        .map(String::from)
        .map_err(ParseError::InvalidField)
}

fn parse_md5_checksum(src: &mut &[u8]) -> Result<Md5Checksum, ParseError> {
    parse_value(src)
        .map_err(ParseError::InvalidField)
        .and_then(|s| s.parse().map_err(ParseError::InvalidMd5Checksum))
}

fn parse_species(src: &mut &[u8]) -> Result<String, ParseError> {
    parse_value(src)
        .map(String::from)
        .map_err(ParseError::InvalidField)
}

fn parse_molecule_topology(src: &mut &[u8]) -> Result<MoleculeTopology, ParseError> {
    parse_value(src)
        .map_err(ParseError::InvalidField)
        .and_then(|s| s.parse().map_err(ParseError::InvalidMoleculeTopology))
}

fn parse_uri(src: &mut &[u8]) -> Result<String, ParseError> {
    parse_value(src)
        .map(String::from)
        .map_err(ParseError::InvalidField)
}

fn try_replace<T>(
    option: &mut Option<T>,
    ctx: &Context,
    tag: Tag,
    value: T,
) -> Result<(), ParseError> {
    if option.replace(value).is_some() && !ctx.allow_duplicate_tags() {
        Err(ParseError::DuplicateTag(tag))
    } else {
        Ok(())
    }
}

fn try_insert<V>(
    other_fields: &mut OtherFields<tag::Standard>,
    ctx: &Context,
    tag: map::tag::Other<tag::Standard>,
    value: V,
) -> Result<(), ParseError>
where
    V: Into<String>,
{
    if other_fields.insert(tag, value.into()).is_some() && !ctx.allow_duplicate_tags() {
        Err(ParseError::DuplicateTag(Tag::Other(tag)))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() -> Result<(), map::reference_sequence::name::ParseError> {
        const LN: NonZeroUsize = match NonZeroUsize::new(8) {
            Some(length) => length,
            None => unreachable!(),
        };

        let mut src = &b"\tSN:sq0\tLN:8"[..];
        let ctx = Context::default();
        let actual = parse_reference_sequence(&mut src, &ctx);

        let name = "sq0".parse()?;
        let map = Map::<ReferenceSequence>::new(LN);
        let expected = (name, map);

        assert_eq!(actual, Ok(expected));

        Ok(())
    }

    #[test]
    fn test_parse_header_with_missing_name() {
        let mut src = &b"\tLN:8"[..];
        let ctx = Context::default();
        assert_eq!(
            parse_reference_sequence(&mut src, &ctx),
            Err(ParseError::MissingName)
        );
    }

    #[test]
    fn test_parse_header_with_missing_length() {
        let mut src = &b"\tSN:sq0"[..];
        let ctx = Context::default();
        assert_eq!(
            parse_reference_sequence(&mut src, &ctx),
            Err(ParseError::MissingLength)
        );
    }

    #[test]
    fn test_parse_header_with_invalid_length() {
        let ctx = Context::default();

        let mut src = &b"\tSN:sq0\tLN:NA"[..];
        assert_eq!(
            parse_reference_sequence(&mut src, &ctx),
            Err(ParseError::InvalidLength)
        );

        let mut src = &b"\tSN:sq0\tLN:0"[..];
        assert_eq!(
            parse_reference_sequence(&mut src, &ctx),
            Err(ParseError::InvalidLength)
        );
    }
}
