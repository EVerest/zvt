use encoding::Encoding;
use log::debug;
use thiserror::Error;

pub mod encoding;
pub mod length;

#[derive(Debug, PartialEq, Error)]
pub enum ZVTError {
    #[error("Incomplete data")]
    IncompleteData,

    #[error("The following tags are required, but were missing: {0:?}")]
    MissingRequiredTags(Vec<Tag>),

    #[error("Not implemented")]
    NonImplemented,

    #[error("Unexpected tag: {0:?}")]
    WrongTag(Tag),

    #[error("Duplicate tag: {0:?}")]
    DuplicateTag(Tag),

    #[error("Received an abort {0}")]
    Aborted(u8),
}

pub type ZVTResult<T> = ::std::result::Result<T, ZVTError>;

/// The tag of a field.
///
/// The tag is equivalent to the bmp-number in the Zvt documentation.
#[derive(Debug, PartialEq, Clone)]
pub struct Tag(pub u16);

/// Trait for commands.
///
/// The trait encodes the control fields of an Adpu package.
pub trait ZvtCommand {
    const CLASS: u8;
    const INSTR: u8;
}

pub trait NotZvtCommand {}

/// The implementation for serializing/deserializing a Zvt struct.
///
/// Trait implements the basic logic of the Zvt protocol. Every package consists
/// of up to three fields:
///
/// `<BMP-NUMBER>` `<LENGTH>` `<DATA>`.
///
/// The BMP-NUMBER and LENGTH are optional; The DATA may be encoded in different
/// ways.
///
/// # Parameters:
///
///  * `L`: The trait [length::Length] encodes/decodes the `<LENGTH>` field.
///         Use [length::Empty] to omit the length.
///  * `E`: The trait [encoding::Encoding] encodes/decodes the given data an
///         generates the DATA field. In order to use this trait with custom
///         types, you have to implement the [encoding::Encoding] trait for your type.
///  * `TE`: The trait [encoding::Encoding] which encodes/decodes the [Tag]
///         the `<BMP-NUMBER>` field.
pub trait ZvtSerializerImpl<
    L: length::Length = length::Empty,
    E: encoding::Encoding<Self> = encoding::Default,
    TE: encoding::Encoding<Tag> = encoding::Default,
> where
    Self: Sized,
{
    fn serialize_tagged(&self, tag: Option<Tag>) -> Vec<u8> {
        let mut output = Vec::new();
        if let Some(tag) = tag {
            output = TE::encode(&tag);
        }
        let mut payload = E::encode(self);
        let mut length = L::serialize(payload.len());
        output.append(&mut length);
        output.append(&mut payload);
        output
    }

    fn deserialize_tagged(mut bytes: &[u8], tag: Option<Tag>) -> ZVTResult<(Self, &[u8])> {
        if let Some(desired_tag) = tag {
            let actual_tag;
            (actual_tag, bytes) = TE::decode(bytes)?;
            if actual_tag != desired_tag {
                return Err(ZVTError::WrongTag(actual_tag));
            }
            debug!(
                "found tag: 0x{:x}, remaining bytes after tag: {:x?}",
                actual_tag.0, bytes
            );
        }
        let (length, payload) = L::deserialize(bytes)?;
        debug!("length: {length}, payload.len: {}", payload.len());
        if length > payload.len() {
            return Err(ZVTError::IncompleteData);
        }
        let (data, remainder) = E::decode(&payload[..length])?;

        Ok((data, &payload[length - remainder.len()..]))
    }
}

/// The implementation for serializing/deserializing a optional Zvt fields.
///
/// Optional fields don't generate anything if the data field is [None]. They
/// generate the same output if the data field is provided.
///
/// When deserializing optional fields behave differently depending if a [Tag]
/// is provided or not. If the [Tag] is [None] the deserialization always
/// succeeds returning [None] as the result. If the [Tag] is provided, then the
/// serialization error is propagated.
impl<T, L: length::Length, E: encoding::Encoding<T>, TE: encoding::Encoding<Tag>>
    ZvtSerializerImpl<L, E, TE> for Option<T>
where
    T: ZvtSerializerImpl<L, E, TE>,
{
    fn serialize_tagged(&self, tag: Option<Tag>) -> Vec<u8> {
        // If the data is missing, the tag is also missing.
        match self {
            None => Vec::new(),
            Some(ref data) => <T as ZvtSerializerImpl<L, E, TE>>::serialize_tagged(data, tag),
        }
    }

    fn deserialize_tagged(bytes: &[u8], tag: Option<Tag>) -> ZVTResult<(Self, &[u8])> {
        // If we're deserializing and the tag is a match but the data is still
        // missing, this is an error.
        match &tag {
            Some(_) => match <T as ZvtSerializerImpl<L, E, TE>>::deserialize_tagged(bytes, tag) {
                Err(err) => Err(err),
                Ok(data) => Ok((Some(data.0), data.1)),
            },
            None => match <T as ZvtSerializerImpl<L, E, TE>>::deserialize_tagged(bytes, None) {
                Err(_) => Ok((None, bytes)),
                Ok(data) => Ok((Some(data.0), data.1)),
            },
        }
    }
}

/// The implementation for serializing/deserializing [Vec]
///
/// The Zvt protocol does not define a consistent handling of vectors. However,
/// in most cases it assumes that every element is tagged and not the vector
/// itself. Therefore we provide a default serialization/deserialization which
/// does exactly this.
///
/// The serialization will return an empty vector for an empty input. Otherwise
/// it will tag every element independently and collect the results.
///
/// The deserialization will deserialize the input until a failure occurs -
/// this indicates that there are no more elements in the vector - and return
/// the results. This means that all elements in the vector must be placed
/// consecutively to each other.
impl<T, L: length::Length, E: encoding::Encoding<T>, TE: encoding::Encoding<Tag>>
    ZvtSerializerImpl<L, E, TE> for Vec<T>
where
    T: ZvtSerializerImpl<L, E, TE>,
{
    fn serialize_tagged(&self, tag: Option<Tag>) -> Vec<u8> {
        self.iter()
            .flat_map(|item| {
                <T as ZvtSerializerImpl<L, E, TE>>::serialize_tagged(item, tag.clone())
            })
            .collect()
    }

    fn deserialize_tagged(mut bytes: &[u8], tag: Option<Tag>) -> ZVTResult<(Self, &[u8])> {
        let mut items = Vec::new();

        while let Ok((item, remainder)) =
            <T as ZvtSerializerImpl<L, E, TE>>::deserialize_tagged(bytes, tag.clone())
        {
            items.push(item);
            bytes = remainder;
        }

        Ok((items, bytes))
    }
}

/// Serializes/Deserializes an Zvt packet.
///
/// The trait wraps the ZvtSerializerImpl and allows a simple serialization/
/// deserialization.
pub trait ZvtSerializer: ZvtSerializerImpl
where
    Self: Sized,
    encoding::Default: encoding::Encoding<Self>,
{
    fn zvt_serialize(&self) -> Vec<u8> {
        <Self as ZvtSerializerImpl>::serialize_tagged(self, None)
    }

    fn zvt_deserialize(bytes: &[u8]) -> ZVTResult<(Self, &[u8])> {
        <Self as ZvtSerializerImpl>::deserialize_tagged(bytes, None)
    }
}

/// Serializes/Deserializes an Adpu packet.
impl<T> ZvtSerializer for T
where
    Self: ZvtCommand
        + ZvtSerializerImpl<length::Adpu, encoding::Default, encoding::BigEndian>
        + ZvtSerializerImpl,
    encoding::Default: encoding::Encoding<Self>,
{
    fn zvt_serialize(&self) -> Vec<u8> {
        // Find a more elegant way to express this.
        let tag: Tag = encoding::BigEndian::decode(&[Self::CLASS, Self::INSTR])
            .unwrap()
            .0;
        <Self as ZvtSerializerImpl<length::Adpu, encoding::Default, encoding::BigEndian>>::serialize_tagged(self, Some(tag))
    }

    fn zvt_deserialize(bytes: &[u8]) -> ZVTResult<(Self, &[u8])> {
        let tag: Tag = encoding::BigEndian::decode(&[Self::CLASS, Self::INSTR])
            .unwrap()
            .0;
        <Self as ZvtSerializerImpl<length::Adpu, encoding::Default, encoding::BigEndian>>::deserialize_tagged(bytes, Some(tag))
    }
}

pub trait ZvtParser
where
    Self: Sized,
{
    fn zvt_parse(bytes: &[u8]) -> ZVTResult<Self>;
}
