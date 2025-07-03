use crate::{reader, util::WzMutableKey, Reader, WzNodeArc, WzNodeCast, WzReader};
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum WzStringParseError {
    #[error("Error parsing WzString: {0}")]
    ParseError(#[from] reader::Error),

    #[error("Not a String property")]
    NotStringProperty,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum WzStringType {
    Ascii,
    Unicode,
    Empty,
}

impl Default for WzStringType {
    fn default() -> Self {
        Self::Ascii
    }
}

#[derive(Debug, Clone)]
pub struct WzStringMeta {
    /// string start offset
    pub offset: usize,
    /// string length
    pub length: u32,
    pub string_type: WzStringType,
}

/// `WzString` only hold the string information.
#[derive(Debug, Clone, Default)]
pub struct WzString {
    reader: Arc<WzReader>,
    /// string start offset
    offset: usize,
    /// string length
    length: u32,
    pub string_type: WzStringType,
}

impl WzStringMeta {
    pub fn new(offset: usize, length: u32, string_type: WzStringType) -> Self {
        Self {
            offset,
            length,
            string_type,
        }
    }
    pub fn empty() -> Self {
        Self {
            offset: 0,
            length: 0,
            string_type: WzStringType::Empty,
        }
    }
    pub fn new_ascii(offset: usize, length: u32) -> Self {
        Self {
            offset,
            length,
            string_type: WzStringType::Ascii,
        }
    }
    pub fn new_unicode(offset: usize, length: u32) -> Self {
        Self {
            offset,
            length,
            string_type: WzStringType::Unicode,
        }
    }
}

impl WzString {
    pub fn from_meta(meta: WzStringMeta, reader: &Arc<WzReader>) -> Self {
        Self {
            reader: Arc::clone(reader),
            offset: meta.offset,
            length: meta.length,
            string_type: meta.string_type,
        }
    }
    /// Create a new `WzString` it will encrypt the string with the given iv.
    pub fn from_str(str: &str, iv: [u8; 4]) -> Self {
        let mut mtbkeys = WzMutableKey::from_iv(iv);

        let len;

        let meta_type = if str.is_empty() {
            len = 0;
            WzStringType::Empty
        } else if str.is_ascii() {
            len = str.len();
            WzStringType::Ascii
        } else {
            len = str.chars().count() * 2;
            WzStringType::Unicode
        };

        let encrypted = encrypt_str(&mut mtbkeys, str, &meta_type);

        let mut reader = WzReader::from_bytes(encrypted);

        reader.wz_iv = iv;
        reader.keys = Arc::new(RwLock::new(mtbkeys));

        WzString {
            reader: Arc::new(reader),
            offset: 0,
            length: len as u32,
            string_type: meta_type,
        }
    }
    /// Decode string from wz file.
    pub fn get_string(&self) -> Result<String, WzStringParseError> {
        self.reader
            .resolve_wz_string_meta(&self.string_type, self.offset, self.length as usize)
            .map_err(WzStringParseError::from)
    }
}

/// A helper function to resolve string from `WzNodeArc`.
pub fn resolve_string_from_node(node: &WzNodeArc) -> Result<String, WzStringParseError> {
    node.read()
        .unwrap()
        .try_as_string()
        .ok_or(WzStringParseError::NotStringProperty)
        .and_then(|string| string.get_string())
}

pub(crate) fn encrypt_str(
    keys: &mut WzMutableKey,
    str: &str,
    string_type: &WzStringType,
) -> Vec<u8> {
    match string_type {
        WzStringType::Empty => Vec::new(),
        WzStringType::Unicode => {
            let mut bytes = str.encode_utf16().collect::<Vec<_>>();

            keys.ensure_key_size(bytes.len() * 2).unwrap();

            bytes
                .iter_mut()
                .enumerate()
                .flat_map(|(i, b)| {
                    let key1 = *keys.try_at(i * 2).unwrap_or(&0) as u16;
                    let key2 = *keys.try_at(i * 2 + 1).unwrap_or(&0) as u16;
                    let i = (i + 0xAAAA) as u16;
                    *b ^= i ^ key1 ^ (key2 << 8);

                    b.to_le_bytes().to_vec()
                })
                .collect()
        }
        WzStringType::Ascii => {
            let mut bytes = str.bytes().collect::<Vec<_>>();

            keys.ensure_key_size(bytes.len()).unwrap();

            for (i, b) in bytes.iter_mut().enumerate() {
                let key = keys.try_at(i).unwrap_or(&0);
                let i = (i + 0xAA) as u8;

                *b ^= i ^ key;
            }

            bytes
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for WzString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let str = self.get_string().unwrap_or_default();

        serializer.serialize_str(&str)
    }
}
#[cfg(feature = "serde")]
use serde::de::{self, Deserializer, Visitor};
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for WzString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;
        struct StringVisitor;
        impl Visitor<'_> for StringVisitor {
            type Value = WzString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string to deserialize into WzString")
            }

            fn visit_str<E>(self, value: &str) -> Result<WzString, E>
            where
                E: de::Error,
            {
                Ok(WzString::from_str(value, Default::default()))
            }
        }

        deserializer.deserialize_str(StringVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::WzNode;
    #[cfg(feature = "serde")]
    use serde_json;

    type Result<T> = std::result::Result<T, WzStringParseError>;

    #[cfg(feature = "serde")]
    #[test]
    fn test_wz_string_serde_ascii() {
        let encrypter_reader = WzReader::default();
        let encrypted = encrypter_reader.encrypt_str("test", &WzStringType::Ascii);

        let reader = WzReader::from_buff(&encrypted);

        let string = WzString::from_meta(WzStringMeta::new_ascii(0, 4), &Arc::new(reader));

        let json = serde_json::to_string(&string).unwrap();
        assert_eq!(json, r#""test""#);

        let string: WzString = serde_json::from_str(r#""test""#).unwrap();
        assert_eq!(string.string_type, WzStringType::Ascii);
        assert_eq!(string.get_string().unwrap(), "test");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_wz_string_serde_unicode() {
        let encrypter_reader = WzReader::default();
        let encrypted = encrypter_reader.encrypt_str("測試", &WzStringType::Unicode);

        let reader = WzReader::from_buff(&encrypted);

        let string = WzString::from_meta(WzStringMeta::new_unicode(0, 4), &Arc::new(reader));

        let json = serde_json::to_string(&string).unwrap();
        assert_eq!(json, r#""測試""#);

        let string: WzString = serde_json::from_str(r#""測試""#).unwrap();
        assert_eq!(string.string_type, WzStringType::Unicode);
        assert_eq!(string.get_string().unwrap(), "測試");
    }

    #[test]
    fn test_wz_string_create_empty() -> Result<()> {
        let wz_string = WzString::from_str("", [0, 0, 0, 0]);

        assert_eq!(wz_string.length, 0);
        assert_eq!(wz_string.string_type, WzStringType::Empty);
        assert_eq!(wz_string.get_string()?, "");

        Ok(())
    }

    #[test]
    fn test_wz_string_create_ascii() -> Result<()> {
        let wz_string = WzString::from_str("test", [0, 0, 0, 0]);

        assert_eq!(wz_string.length, 4);
        assert_eq!(wz_string.string_type, WzStringType::Ascii);
        assert_eq!(wz_string.get_string()?, "test");

        Ok(())
    }

    #[test]
    fn test_wz_string_create_unicode() -> Result<()> {
        let wz_string = WzString::from_str("測試", [0, 0, 0, 0]);

        assert_eq!(wz_string.length, 4);
        assert_eq!(wz_string.string_type, WzStringType::Unicode);
        assert_eq!(wz_string.get_string()?, "測試");

        Ok(())
    }

    #[test]
    fn test_resolve_from_node_success() -> Result<()> {
        let node =
            WzNode::from_str("root", WzString::from_str("test", [0, 0, 0, 0]), None).into_lock();

        assert_eq!(resolve_string_from_node(&node)?, "test");

        Ok(())
    }

    #[test]
    fn test_resolve_from_node_fail() -> Result<()> {
        let node = WzNode::from_str("root", 1, None).into_lock();

        assert!(resolve_string_from_node(&node).is_err());

        Ok(())
    }
}
