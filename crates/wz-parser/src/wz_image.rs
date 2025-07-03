use crate::property::WzLua;
use crate::version::{guess_iv_from_wz_img, verify_iv_from_wz_img};
use crate::{reader, util, WzNode, WzNodeArc, WzNodeArcVec, WzNodeName, WzReader};
use std::sync::Arc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FileError(#[from] std::io::Error),
    #[error("Lua parse error")]
    LuaParseError,
    #[error("parse as wz image failed, pos at {0}")]
    ParseError(usize),
    #[error("New Wz image header found. b = {0}, offset = {1}")]
    UnknownImageHeader(u8, usize),
    #[error("Wrong Wz Version")]
    WrongVersion,
    #[error("Unable to guess version")]
    UnableToGuessVersion,
    #[error(transparent)]
    ParsePropertyListError(#[from] util::WzPropertyParseError),
    #[error("Binary reading error")]
    ReaderError(#[from] reader::Error),
    #[error("Not a Image object")]
    NotImageObject,
}

pub const WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET: u8 = 0x73;
pub const WZ_IMAGE_HEADER_BYTE_WITH_OFFSET: u8 = 0x1B;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Debug, Clone, Default)]
pub struct WzImage {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub reader: Arc<WzReader>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub name: WzNodeName,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub offset: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub block_size: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub is_parsed: bool,
}

impl WzImage {
    pub fn new(
        name: &WzNodeName,
        offset: usize,
        block_size: usize,
        reader: &Arc<WzReader>,
    ) -> Self {
        Self {
            reader: Arc::clone(reader),
            name: name.clone(),
            offset,
            block_size,
            is_parsed: false,
        }
    }
    pub fn from_file<P>(path: P, wz_iv: Option<[u8; 4]>) -> Result<Self, Error>
    where
        P: AsRef<std::path::Path>,
    {
        let name = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let map = std::fs::read(path)?;

        let wz_iv = if let Some(iv) = wz_iv {
            if !verify_iv_from_wz_img(&map, &iv) {
                return Err(Error::WrongVersion);
            }
            iv
        } else {
            guess_iv_from_wz_img(&map).ok_or(Error::UnableToGuessVersion)?
        };

        let block_size = map.len();
        let reader = WzReader::new(map).with_iv(wz_iv);

        Ok(WzImage {
            reader: Arc::new(reader),
            name: name.into(),
            offset: 0,
            block_size,
            is_parsed: false,
        })
    }

    /// Direct get child node inside `WzImage` without parsing the whole `WzImage`. Sometimes
    /// we just need a single node in `WzImage`, but don't want to parse it and
    /// unparse later, it waste time and memory.
    pub fn at_path(&self, path: &str) -> Result<WzNodeArc, Error> {
        let reader = self.reader.create_slice_reader_without_hash();

        reader.seek(self.offset);
        let header_byte = reader.read_u8()?;

        if header_byte != WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET {
            return Err(Error::UnknownImageHeader(header_byte, reader.pos.get()));
        } else {
            let name = reader.read_wz_string()?;
            let value = reader.read_u16()?;
            if name != "Property" && value != 0 {
                return Err(Error::ParseError(reader.pos.get()));
            }
        }

        let result = util::get_node(path, &self.reader, &reader, self.offset);

        match result {
            Ok(node) => Ok(node.1),
            Err(e) => Err(Error::from(e)),
        }
    }

    /// Parse the whole `WzImage` and return all children nodes. It can be directly used when you want to keep the original order of children.
    pub fn resolve_children(
        &self,
        parent: Option<&WzNodeArc>,
    ) -> Result<(WzNodeArcVec, Vec<WzNodeArc>), Error> {
        let reader = self.reader.create_slice_reader_without_hash();

        reader.seek(self.offset);

        let header_byte = reader.read_u8()?;

        match header_byte {
            0x1 => {
                if self.name.ends_with(".lua") {
                    let len = reader.read_wz_int()?;
                    let offset = reader.pos.get();

                    let name: WzNodeName = String::from("Script").into();

                    let wz_lua = WzLua::new(&self.reader, offset, len as usize);

                    let lua_node = WzNode::new(&name, wz_lua, parent);

                    return Ok((vec![(name, lua_node.into_lock())], vec![]));
                }
                return Err(Error::LuaParseError);
            }
            WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET => {
                let name = reader.read_wz_string()?;
                let value = reader.read_u16()?;
                if name != "Property" && value != 0 {
                    return Err(Error::WrongVersion);
                }
            }
            _ => {
                return Err(Error::UnknownImageHeader(header_byte, reader.pos.get()));
            }
        }

        util::parse_property_list(parent, &self.reader, &reader, self.offset).map_err(Error::from)
    }
}

pub fn is_lua_image(name: &str) -> bool {
    name.ends_with(".lua")
}
pub fn is_valid_wz_image(check_byte: u8) -> bool {
    check_byte == WZ_IMAGE_HEADER_BYTE_WITH_OFFSET
        || check_byte == WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET
}
