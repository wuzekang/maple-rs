use crate::property::{
    Vector2D, WzLua, WzPng, WzRawData, WzSound, WzString, WzSubProperty, WzValue,
};
use crate::{WzDirectory, WzFile, WzImage};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// All variants of `WzObjectType`.
///
/// `WzObjectType` implement most of the From trait for the types that can be converted to it.
///
/// # Example
///
/// ```
/// # use wz_parser::WzObjectType;
/// # use wz_parser::property::WzValue;
/// let wz_int: WzObjectType = 1.into();
///
/// assert!(matches!(wz_int, WzObjectType::Value(WzValue::Int(1))));
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[derive(Debug, Clone)]
pub enum WzObjectType {
    File(Box<WzFile>),
    Image(Box<WzImage>),
    Directory(Box<WzDirectory>),
    #[cfg_attr(feature = "serde", serde(untagged))]
    Property(WzSubProperty),
    #[cfg_attr(feature = "serde", serde(untagged))]
    Value(WzValue),
}

macro_rules! from_impl_wz_files {
    ($from_type:ident, $ot_variant:ident) => {
        impl From<$from_type> for WzObjectType {
            fn from(i: $from_type) -> Self {
                WzObjectType::$ot_variant(Box::new(i))
            }
        }
    };
}
macro_rules! from_impl_wz_value {
    ($from_type:ident, $variant:ident) => {
        impl From<$from_type> for WzObjectType {
            fn from(i: $from_type) -> Self {
                WzObjectType::Value(WzValue::$variant(i))
            }
        }
    };
}
macro_rules! from_impl_wz_property {
    ($from_type:ident, $variant:ident) => {
        impl From<$from_type> for WzObjectType {
            fn from(i: $from_type) -> Self {
                WzObjectType::Property(WzSubProperty::$variant(Box::new(i)))
            }
        }
    };
}

from_impl_wz_files!(WzFile, File);
from_impl_wz_files!(WzDirectory, Directory);
from_impl_wz_files!(WzImage, Image);

from_impl_wz_value!(i16, Short);
from_impl_wz_value!(i32, Int);
from_impl_wz_value!(i64, Long);
from_impl_wz_value!(f32, Float);
from_impl_wz_value!(f64, Double);
from_impl_wz_value!(WzString, String);
from_impl_wz_value!(Vector2D, Vector);
from_impl_wz_value!(WzRawData, RawData);
from_impl_wz_value!(WzLua, Lua);

from_impl_wz_property!(WzPng, PNG);
from_impl_wz_property!(WzSound, Sound);

#[cfg(feature = "serde")]
#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "serde")]
    

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_main_wz_object() {
        let file: WzObjectType = WzFile::default().into();
        let dir: WzObjectType = WzDirectory::default().into();
        let image: WzObjectType = WzImage::default().into();

        let file_json = serde_json::to_string(&file).unwrap();
        let dir_json = serde_json::to_string(&dir).unwrap();
        let image_json = serde_json::to_string(&image).unwrap();

        assert_eq!(
            file_json,
            r#"{"type":"File","data":{"path":"","patch_version":0,"wz_version_header":0,"wz_with_encrypt_version_header":false,"hash":0}}"#
        );
        assert_eq!(dir_json, r#"{"type":"Directory","data":{}}"#);
        assert_eq!(image_json, r#"{"type":"Image","data":{}}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_sub_and_value_wz_object() {
        let sub: WzObjectType = WzPng::default().into();
        let value: WzObjectType = 1.into();

        let sub_json = serde_json::to_string(&sub).unwrap();
        let value_json = serde_json::to_string(&value).unwrap();

        // not need to test every property, just need to check is actually using untagged
        assert_eq!(sub_json, r#"{"type":"PNG","data":{"width":0,"height":0}}"#);
        assert_eq!(value_json, r#"{"type":"Int","data":1}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_main_wz_object() {
        let file_json = r#"{"type":"File","data":{"path":"","patch_version":0,"wz_version_header":0,"wz_with_encrypt_version_header":false,"hash":0}}"#;
        let dir_json = r#"{"type":"Directory","data":{}}"#;
        let image_json = r#"{"type":"Image","data":{}}"#;

        let file: WzObjectType = serde_json::from_str(file_json).unwrap();
        let dir: WzObjectType = serde_json::from_str(dir_json).unwrap();
        let image: WzObjectType = serde_json::from_str(image_json).unwrap();

        assert!(matches!(file, WzObjectType::File(_)));
        assert!(matches!(dir, WzObjectType::Directory(_)));
        assert!(matches!(image, WzObjectType::Image(_)));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_sub_and_value_wz_object() {
        let sub_json = r#"{"type":"PNG","data":{"width":0,"height":0}}"#;
        let value_json = r#"{"type":"Int","data":1}"#;

        let sub: WzObjectType = serde_json::from_str(sub_json).unwrap();
        let value: WzObjectType = serde_json::from_str(value_json).unwrap();

        assert!(matches!(sub, WzObjectType::Property(_)));
        assert!(matches!(value, WzObjectType::Value(_)));
    }
}
