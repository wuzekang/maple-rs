#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use serde_json::{Number, Value};

pub mod lua;
pub mod png;
pub mod raw_data;
pub mod sound;
pub mod string;
pub mod vector;

pub use lua::*;
pub use png::*;
pub use raw_data::*;
pub use sound::*;
pub use string::*;
pub use vector::*;

// #[derive(Debug, Clone)]
// pub enum WzPropertyType {
//   Null,
//   Short(i16),
//   Int(i32),
//   Long(i64),
//   Float(f32),
//   Double(f64),
//   String(WzStringType),

//   SubProperty,
//   Vector(Vector2D),
//   Convex,
//   Sound(WzSoundMeta),
//   UOL(WzStringType),
//   Lua,

//   PNG(WzPng),

//   RawData,
// }

/// A WzProperty potentially contains childrens.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
pub enum WzSubProperty {
    Convex,
    Sound(Box<WzSound>),
    PNG(Box<WzPng>),
    #[cfg_attr(feature = "serde", serde(other))]
    Property,
}

/// Some basic value, more like a primitive type.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[derive(Debug, Clone)]
pub enum WzValue {
    #[cfg_attr(feature = "serde", serde(skip))]
    RawData(WzRawData),
    #[cfg_attr(feature = "serde", serde(skip))]
    Lua(WzLua),
    Short(i16),
    #[cfg_attr(feature = "serde", serde(alias = "number"))]
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Vector(Vector2D),
    UOL(WzString),
    String(WzString),
    ParsedString(String),
    #[cfg_attr(feature = "serde", serde(other))]
    Null,
}

impl Default for WzValue {
    fn default() -> Self {
        Self::Null
    }
}

#[cfg(feature = "json")]
impl From<WzValue> for Value {
    fn from(value: WzValue) -> Self {
        match value {
            WzValue::Null => Value::Null,
            WzValue::RawData(_) => Value::Null,
            WzValue::Lua(_) => Value::Null,
            WzValue::Short(value) => value.into(),
            WzValue::Int(value) => value.into(),
            WzValue::Long(value) => value.into(),
            WzValue::Float(value) => Value::Number(Number::from_f64(value.into()).unwrap()),
            WzValue::Double(value) => Value::Number(Number::from_f64(value).unwrap()),
            WzValue::Vector(Vector2D(x, y)) => {
                let mut vec = serde_json::Map::new();
                vec.insert("x".to_string(), x.into());
                vec.insert("y".to_string(), y.into());
                Value::Object(vec)
            }
            WzValue::UOL(string) | WzValue::String(string) => {
                string.get_string().unwrap_or_default().into()
            }
            WzValue::ParsedString(string) => string.into(),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "serde")]
    use serde_json;

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_wz_sub_property() {
        let png = WzSubProperty::PNG(Box::default());
        let sound = WzSubProperty::Sound(Box::default());
        let property = WzSubProperty::Property;
        let convex = WzSubProperty::Convex;

        let png_json = serde_json::to_string(&png).unwrap();
        let sound_json = serde_json::to_string(&sound).unwrap();
        let property_json = serde_json::to_string(&property).unwrap();
        let convex_json = serde_json::to_string(&convex).unwrap();

        assert_eq!(png_json, r#"{"type":"PNG","data":{"width":0,"height":0}}"#);
        assert_eq!(
            sound_json,
            r#"{"type":"Sound","data":{"duration":0,"sound_type":"Binary"}}"#
        );
        assert_eq!(property_json, r#"{"type":"Property"}"#);
        assert_eq!(convex_json, r#"{"type":"Convex"}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_wz_sub_property() {
        let png_json = r#"{"type":"PNG","data":{"width":0,"height":0}}"#;
        let sound_json = r#"{"type":"Sound","data":{"duration":0,"sound_type":"Binary"}}"#;
        let property_json = r#"{"type":"Property"}"#;
        let convex_json = r#"{"type":"Convex"}"#;

        let png: WzSubProperty = serde_json::from_str(png_json).unwrap();
        let sound: WzSubProperty = serde_json::from_str(sound_json).unwrap();
        let property: WzSubProperty = serde_json::from_str(property_json).unwrap();
        let convex: WzSubProperty = serde_json::from_str(convex_json).unwrap();

        assert!(matches!(png, WzSubProperty::PNG(_)));
        assert!(matches!(sound, WzSubProperty::Sound(_)));
        assert!(matches!(property, WzSubProperty::Property));
        assert!(matches!(convex, WzSubProperty::Convex));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_wz_value() {
        let null = WzValue::Null;
        let raw_data = WzValue::RawData(WzRawData::default());
        let lua = WzValue::Lua(WzLua::default());
        let short = WzValue::Short(1);
        let int = WzValue::Int(1);
        let long = WzValue::Long(1);
        let float = WzValue::Float(1.1);
        let double = WzValue::Double(1.1);
        let vector = WzValue::Vector(Vector2D(1, 1));
        let uol = WzValue::UOL(WzString::from_str("1/1", [0, 0, 0, 0]));
        let string = WzValue::String(WzString::from_str("string", [0, 0, 0, 0]));
        let parsed_string = WzValue::ParsedString("string".to_string());

        let null_json = serde_json::to_string(&null).unwrap();
        assert!(serde_json::to_string(&raw_data).is_err());
        assert!(serde_json::to_string(&lua).is_err());
        let short_json = serde_json::to_string(&short).unwrap();
        let int_json = serde_json::to_string(&int).unwrap();
        let long_json = serde_json::to_string(&long).unwrap();
        let float_json = serde_json::to_string(&float).unwrap();
        let double_json = serde_json::to_string(&double).unwrap();
        let vector_json = serde_json::to_string(&vector).unwrap();
        let uol_json = serde_json::to_string(&uol).unwrap();
        let string_json = serde_json::to_string(&string).unwrap();
        let parsed_string_json = serde_json::to_string(&parsed_string).unwrap();

        assert_eq!(null_json, r#"{"type":"Null"}"#);
        assert_eq!(short_json, r#"{"type":"Short","data":1}"#);
        assert_eq!(int_json, r#"{"type":"Int","data":1}"#);
        assert_eq!(long_json, r#"{"type":"Long","data":1}"#);
        assert_eq!(float_json, r#"{"type":"Float","data":1.1}"#);
        assert_eq!(double_json, r#"{"type":"Double","data":1.1}"#);
        assert_eq!(vector_json, r#"{"type":"Vector","data":[1,1]}"#);
        assert_eq!(uol_json, r#"{"type":"UOL","data":"1/1"}"#);
        assert_eq!(string_json, r#"{"type":"String","data":"string"}"#);
        assert_eq!(
            parsed_string_json,
            r#"{"type":"ParsedString","data":"string"}"#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_wz_value() {
        let null_json = r#"{"type":"Null"}"#;
        let raw_data_json = r#"{"type":"RawData"}"#;
        let lua_json = r#"{"type":"Lua"}"#;
        let short_json = r#"{"type":"Short","data":1}"#;
        let int_json = r#"{"type":"Int","data":1}"#;
        let long_json = r#"{"type":"Long","data":1}"#;
        let float_json = r#"{"type":"Float","data":1.0}"#;
        let double_json = r#"{"type":"Double","data":1.0}"#;
        let vector_json = r#"{"type":"Vector", "data": [1, 1]}"#;
        let uol_json = r#"{"type":"UOL","data":"1/1"}"#;
        let string_json = r#"{"type":"String","data":"string"}"#;
        let parsed_string_json = r#"{"type":"ParsedString","data":"string"}"#;

        let null: WzValue = serde_json::from_str(null_json).unwrap();
        let null_1: WzValue = serde_json::from_str(raw_data_json).unwrap();
        let null_2: WzValue = serde_json::from_str(lua_json).unwrap();
        let short: WzValue = serde_json::from_str(short_json).unwrap();
        let int: WzValue = serde_json::from_str(int_json).unwrap();
        let long: WzValue = serde_json::from_str(long_json).unwrap();
        let float: WzValue = serde_json::from_str(float_json).unwrap();
        let double: WzValue = serde_json::from_str(double_json).unwrap();
        let vector: WzValue = serde_json::from_str(vector_json).unwrap();
        let uol: WzValue = serde_json::from_str(uol_json).unwrap();
        let string: WzValue = serde_json::from_str(string_json).unwrap();
        let parsed_string: WzValue = serde_json::from_str(parsed_string_json).unwrap();

        assert!(matches!(null, WzValue::Null));
        assert!(matches!(null_1, WzValue::Null));
        assert!(matches!(null_2, WzValue::Null));
        assert!(matches!(short, WzValue::Short(1)));
        assert!(matches!(int, WzValue::Int(1)));
        assert!(matches!(long, WzValue::Long(1)));
        assert!(matches!(float, WzValue::Float(_)));
        assert!(matches!(double, WzValue::Double(_)));
        assert!(matches!(vector, WzValue::Vector(Vector2D(1, 1))));
        assert!(matches!(uol, WzValue::UOL(_)));
        assert!(matches!(string, WzValue::String(_)));
        assert!(matches!(parsed_string, WzValue::ParsedString(_)));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_from_wz_value_to_serde_json_value() {
        let null = WzValue::Null;
        let raw_data = WzValue::RawData(WzRawData::default());
        let lua = WzValue::Lua(WzLua::default());
        let short = WzValue::Short(1);
        let int = WzValue::Int(1);
        let long = WzValue::Long(1);
        let float = WzValue::Float(1.1);
        let double = WzValue::Double(1.1);
        let vector = WzValue::Vector(Vector2D(1, 1));
        let uol = WzValue::UOL(WzString::from_str("1/1", [0, 0, 0, 0]));
        let string = WzValue::String(WzString::from_str("string", [0, 0, 0, 0]));
        let parsed_string = WzValue::ParsedString("string".to_string());

        let null_json: Value = null.into();
        let raw_data_json: Value = raw_data.into();
        let lua_json: Value = lua.into();
        let short_json: Value = short.into();
        let int_json: Value = int.into();
        let long_json: Value = long.into();
        let float_json: Value = float.into();
        let double_json: Value = double.into();
        let vector_json: Value = vector.into();
        let uol_json: Value = uol.into();
        let string_json: Value = string.into();
        let parsed_string_json: Value = parsed_string.into();

        assert_eq!(null_json, Value::Null);
        assert_eq!(raw_data_json, Value::Null);
        assert_eq!(lua_json, Value::Null);
        assert_eq!(short_json, Value::Number(Number::from(1)));
        assert_eq!(int_json, Value::Number(Number::from(1)));
        assert_eq!(long_json, Value::Number(Number::from(1)));
        assert_eq!(
            float_json,
            Value::Number(Number::from_f64((1.1_f32).into()).unwrap())
        );
        assert_eq!(double_json, Value::Number(Number::from_f64(1.1).unwrap()));
        assert_eq!(
            vector_json,
            Value::Object({
                let mut map = serde_json::Map::new();
                map.insert("x".to_string(), Value::Number(Number::from(1)));
                map.insert("y".to_string(), Value::Number(Number::from(1)));
                map
            })
        );
        assert_eq!(uol_json, Value::String("1/1".to_string()));
        assert_eq!(string_json, Value::String("string".to_string()));
        assert_eq!(parsed_string_json, Value::String("string".to_string()));
    }
}
