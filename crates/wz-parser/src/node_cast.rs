use crate::property::{
    Vector2D, WzLua, WzPng, WzRawData, WzSound, WzString, WzSubProperty, WzValue,
};
use crate::{WzDirectory, WzFile, WzImage, WzNode, WzObjectType};

/// Trait for casting `WzNode` to its inner type.
///
/// # Example
///
/// ```
/// # use wz_parser::{WzNode, WzNodeCast};
/// let wz_int = WzNode::from_str("test", 1, None);
///
/// assert!(wz_int.try_as_int().is_some());
/// assert!(wz_int.try_as_file().is_none());
/// ```
pub trait WzNodeCast {
    fn try_as_file(&self) -> Option<&WzFile>;
    fn try_as_directory(&self) -> Option<&WzDirectory>;
    fn try_as_image(&self) -> Option<&WzImage>;

    fn try_as_sub_property(&self) -> Option<&WzSubProperty>;
    fn try_as_value(&self) -> Option<&WzValue>;

    fn try_as_png(&self) -> Option<&WzPng>;
    fn try_as_sound(&self) -> Option<&WzSound>;
    fn try_as_string(&self) -> Option<&WzString>;
    fn is_sub_property(&self) -> bool;
    fn is_convex(&self) -> bool;

    fn try_as_lua(&self) -> Option<&WzLua>;
    fn try_as_raw_data(&self) -> Option<&WzRawData>;

    fn is_null(&self) -> bool;
    fn try_as_vector2d(&self) -> Option<&Vector2D>;
    fn try_as_short(&self) -> Option<&i16>;
    fn try_as_int(&self) -> Option<&i32>;
    fn try_as_long(&self) -> Option<&i64>;
    fn try_as_float(&self) -> Option<&f32>;
    fn try_as_double(&self) -> Option<&f64>;
    fn try_as_uol(&self) -> Option<&WzString>;
}

macro_rules! try_as {
    ($func_name:ident, $variant:ident, $result:ty) => {
        fn $func_name(&self) -> Option<&$result> {
            match &self.object_type {
                WzObjectType::$variant(inner) => Some(inner),
                _ => None,
            }
        }
    };
}

macro_rules! try_as_wz_value {
    ($func_name:ident, $variant:ident, $result:ident) => {
        fn $func_name(&self) -> Option<&$result> {
            match &self.object_type {
                WzObjectType::Value(WzValue::$variant(inner)) => Some(inner),
                _ => None,
            }
        }
    };
}

impl WzNodeCast for WzNode {
    try_as!(try_as_file, File, WzFile);
    try_as!(try_as_directory, Directory, WzDirectory);
    try_as!(try_as_image, Image, WzImage);

    try_as!(try_as_sub_property, Property, WzSubProperty);
    try_as!(try_as_value, Value, WzValue);

    fn try_as_png(&self) -> Option<&WzPng> {
        match &self.object_type {
            WzObjectType::Property(WzSubProperty::PNG(png)) => Some(png),
            _ => None,
        }
    }
    fn try_as_sound(&self) -> Option<&WzSound> {
        match &self.object_type {
            WzObjectType::Property(WzSubProperty::Sound(sound)) => Some(sound),
            _ => None,
        }
    }
    fn try_as_string(&self) -> Option<&WzString> {
        match &self.object_type {
            WzObjectType::Value(WzValue::String(string))
            | WzObjectType::Value(WzValue::UOL(string)) => Some(string),
            _ => None,
        }
    }

    fn is_sub_property(&self) -> bool {
        matches!(
            &self.object_type,
            WzObjectType::Property(WzSubProperty::Property)
        )
    }
    fn is_convex(&self) -> bool {
        matches!(
            &self.object_type,
            WzObjectType::Property(WzSubProperty::Convex)
        )
    }
    fn is_null(&self) -> bool {
        matches!(&self.object_type, WzObjectType::Value(WzValue::Null))
    }

    try_as_wz_value!(try_as_lua, Lua, WzLua);
    try_as_wz_value!(try_as_raw_data, RawData, WzRawData);

    try_as_wz_value!(try_as_vector2d, Vector, Vector2D);
    try_as_wz_value!(try_as_short, Short, i16);
    try_as_wz_value!(try_as_int, Int, i32);
    try_as_wz_value!(try_as_long, Long, i64);
    try_as_wz_value!(try_as_float, Float, f32);
    try_as_wz_value!(try_as_double, Double, f64);
    try_as_wz_value!(try_as_uol, UOL, WzString);
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::property::{WzSoundType, WzStringMeta};
    use crate::WzReader;
    use std::sync::Arc;

    fn setup_wz_reader() -> Result<WzReader, std::io::Error> {
        let data = vec![0u8; 200];

        Ok(WzReader::from_bytes(data))
    }

    #[test]
    fn try_as_file() {
        let reader = setup_wz_reader().unwrap();
        let file = WzFile {
            offset: 0,
            block_size: 0,
            is_parsed: false,
            reader: Arc::new(reader),
            wz_file_meta: Default::default(),
        };
        let node = WzNode::from_str("test", file, None);

        assert!(node.try_as_file().is_some());
        assert!(node.try_as_directory().is_none());
    }

    #[test]
    fn try_as_directory() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let wzdir = WzDirectory::new(0, 0, &reader, false);
        let node = WzNode::from_str("test", wzdir, None);

        assert!(node.try_as_directory().is_some());
        assert!(node.try_as_file().is_none());
    }

    #[test]
    fn try_as_image() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let wzimage = WzImage::new(&"test".into(), 0, 0, &reader);
        let node = WzNode::from_str("test", wzimage, None);

        assert!(node.try_as_image().is_some());
        assert!(node.try_as_file().is_none());
    }

    #[test]
    fn try_as_sub_property() {
        let node = WzNode::from_str(
            "test",
            WzObjectType::Property(WzSubProperty::Property),
            None,
        );

        assert!(node.try_as_sub_property().is_some());
        assert!(node.try_as_file().is_none());
    }
    #[test]
    fn try_as_value() {
        let node = WzNode::from_str("test", WzObjectType::Value(WzValue::Null), None);

        assert!(node.try_as_value().is_some());
        assert!(node.try_as_file().is_none());
    }

    #[test]
    fn try_as_png() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let png = WzPng::new(&reader, (1, 1), (1, 1), (0, 1), 0);
        let node = WzNode::from_str("test", png, None);

        assert!(node.try_as_png().is_some());
        assert!(node.try_as_file().is_none());
    }
    #[test]
    fn try_as_sound() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let sound = WzSound::new(&reader, 0, 0, 0, 0, 0, WzSoundType::Mp3);
        let node = WzNode::from_str("test", sound, None);

        assert!(node.try_as_sound().is_some());
        assert!(node.try_as_file().is_none());
    }
    #[test]
    fn try_as_string() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let string = WzString::from_meta(WzStringMeta::empty(), &reader);
        let node = WzNode::from_str("test", string, None);

        assert!(node.try_as_string().is_some());
        assert!(node.try_as_file().is_none());
    }

    #[test]
    fn try_as_string_uol() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let string = WzString::from_meta(WzStringMeta::empty(), &reader);
        let node = WzNode::from_str("test", WzObjectType::Value(WzValue::UOL(string)), None);

        assert!(node.try_as_string().is_some());
        assert!(node.try_as_file().is_none());
    }

    #[test]
    fn try_as_uol() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let string = WzString::from_meta(WzStringMeta::empty(), &reader);
        let node = WzNode::from_str("test", WzObjectType::Value(WzValue::UOL(string)), None);

        assert!(node.try_as_uol().is_some());
        assert!(node.try_as_file().is_none());
    }

    #[test]
    fn is_sub_property() {
        let node = WzNode::from_str(
            "test",
            WzObjectType::Property(WzSubProperty::Property),
            None,
        );

        assert!(node.is_sub_property());
        assert!(!node.is_convex());
    }
    #[test]
    fn is_convex() {
        let node = WzNode::from_str("test", WzObjectType::Property(WzSubProperty::Convex), None);

        assert!(node.is_convex());
        assert!(!node.is_sub_property());
    }

    #[test]
    fn is_null() {
        let node = WzNode::from_str("test", WzObjectType::Value(WzValue::Null), None);

        assert!(node.is_null());
        assert!(!node.is_sub_property());
    }

    #[test]
    fn try_as_lua() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let lua = WzLua::new(&reader, 0, 0);
        let node = WzNode::from_str("test", lua, None);

        assert!(node.try_as_lua().is_some());
        assert!(node.try_as_file().is_none());
    }
    #[test]
    fn try_as_raw_data() {
        let reader = Arc::new(setup_wz_reader().unwrap());
        let raw_data = WzRawData::new(&reader, 0, 0);
        let node = WzNode::from_str("test", raw_data, None);

        assert!(node.try_as_raw_data().is_some());
        assert!(node.try_as_file().is_none());
    }

    #[test]
    fn try_as_vector2d() {
        let vec2 = Vector2D::new(2, 3);
        let node = WzNode::from_str("test", vec2, None);

        assert!(node.try_as_file().is_none());
        assert_eq!(node.try_as_vector2d(), Some(&Vector2D::new(2, 3)));
    }
    #[test]
    fn try_as_short() {
        let node = WzNode::from_str("test", 1_i16, None);

        assert!(node.try_as_file().is_none());
        assert_eq!(node.try_as_short(), Some(&1));
    }
    #[test]
    fn try_as_int() {
        let node = WzNode::from_str("test", 1, None);

        assert!(node.try_as_file().is_none());
        assert_eq!(node.try_as_int(), Some(&1));
    }
    #[test]
    fn try_as_long() {
        let node = WzNode::from_str("test", 1_i64, None);

        assert!(node.try_as_file().is_none());
        assert_eq!(node.try_as_long(), Some(&1));
    }
    #[test]
    fn try_as_float() {
        let node = WzNode::from_str("test", 1.0_f32, None);

        assert!(node.try_as_file().is_none());
        assert_eq!(node.try_as_float(), Some(&1.0));
    }
    #[test]
    fn try_as_double() {
        let node = WzNode::from_str("test", 1.0_f64, None);

        assert!(node.try_as_file().is_none());
        assert_eq!(node.try_as_double(), Some(&1.0));
    }
}
