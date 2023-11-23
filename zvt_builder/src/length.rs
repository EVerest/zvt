use super::encoding::{Default, Encoding};
use super::*;

pub trait Length {
    fn serialize(len: usize) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> ZVTResult<(usize, &[u8])>;
}

/// Marker for the case where we don't encode the length.
///
/// This length type is applied by default. It does not serialize anything
/// and does not consume any data.
pub struct Empty;

impl Length for Empty {
    fn serialize(_: usize) -> Vec<u8> {
        vec![]
    }

    fn deserialize(bytes: &[u8]) -> ZVTResult<(usize, &[u8])> {
        Ok((bytes.len(), bytes))
    }
}

/// Marker for fixed length.
///
/// A fixed length pads the input to the desired length. The deserialization
/// will always return the fixed value and the full data.
pub struct Fixed<const N: usize>(pub usize);

impl<const N: usize> Length for Fixed<N> {
    fn serialize(len: usize) -> Vec<u8> {
        vec![0; N - len]
    }

    fn deserialize(data: &[u8]) -> ZVTResult<(usize, &[u8])> {
        if data.len() < N {
            return Err(ZVTError::IncompleteData);
        }
        Ok((N, data))
    }
}

/// Marker for Tlv data types.
pub struct Tlv;

impl Length for Tlv {
    fn serialize(len: usize) -> Vec<u8> {
        const U8_MAX: usize = u8::MAX as usize;
        const U16_MAX: usize = u16::MAX as usize;

        match len {
            0..=127 => vec![len as u8],
            128..=U8_MAX => vec![0x81, len as u8],
            256..=U16_MAX => {
                let bytes = (len as u16).to_be_bytes().to_vec();
                [vec![0x82], bytes].concat()
            }
            _ => panic!("Unsupported length"),
        }
    }

    fn deserialize(data: &[u8]) -> ZVTResult<(usize, &[u8])> {
        let Some(d) = data.first()
        else {
            return Err(ZVTError::IncompleteData);
        };

        match d {
            0..=127 => Ok((*d as usize, &data[1..])),
            0x81 => {
                if let Some(d) = data.get(1) {
                    Ok((*d as usize, &data[2..]))
                } else {
                    Err(ZVTError::IncompleteData)
                }
            }
            0x82 => {
                let bytes: [u8; 2] = data[1..3]
                    .try_into()
                    .map_err(|_| ZVTError::IncompleteData)?;
                Ok((u16::from_be_bytes(bytes) as usize, &data[3..]))
            }
            _ => Err(ZVTError::NonImplemented),
        }
    }
}

/// The LLV format is just uncompressed BCD.
pub struct LlvImpl<const N: usize>;

impl<const N: usize> Length for LlvImpl<N> {
    fn serialize(input: usize) -> Vec<u8> {
        let mut k = input;
        let mut rv = vec![0; N];
        for i in (0..N).rev() {
            rv[i] = (k % 10) as u8;
            k /= 10;
        }
        rv
    }

    fn deserialize(data: &[u8]) -> ZVTResult<(usize, &[u8])> {
        let mut rv = 0;
        for i in 0..N {
            let Some(d) = data.get(i)
            else {
                return Err(ZVTError::IncompleteData);
            };
            let d = (d & 0xf) as usize;
            rv = rv * 10 + d;
        }
        Ok((rv, &data[N..]))
    }
}

pub type Llv = LlvImpl<2>;
pub type Lllv = LlvImpl<3>;

pub struct Adpu;

impl Length for Adpu {
    fn serialize(input: usize) -> Vec<u8> {
        if input < 0xff {
            vec![input as u8]
        } else {
            let res = Default::encode(&(input as u16));
            [vec![0xff], res].concat()
        }
    }

    fn deserialize(data: &[u8]) -> ZVTResult<(usize, &[u8])> {
        let Some(d) = data.first()
        else {
            return Err(ZVTError::IncompleteData);
        };

        if *d == 0xff {
            let res: (u16, _) = Default::decode(&data[1..])?;
            Ok((res.0 as usize, res.1))
        } else {
            Ok((*d as usize, &data[1..]))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_tlv() {
        let data = vec![0, 127, 255, 256, 300];

        for d in data {
            let bytes = Tlv::serialize(d as usize);
            let (output, _) = Tlv::deserialize(&bytes).unwrap();
            assert_eq!(d, output, "{:?}", bytes);
        }
    }
}
