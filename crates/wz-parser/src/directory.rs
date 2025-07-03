use crate::{
    reader, Reader, WzImage, WzNode, WzNodeArc, WzNodeArcVec, WzNodeName, WzObjectType, WzReader,
};
use std::sync::Arc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lua parse error")]
    LuaParseError,
    #[error("parse as wz image failed, pos at {0}")]
    ParseError(usize),
    #[error("New Wz image header found. b = {0}, offset = {1}")]
    UnknownWzDirectoryType(u8, usize),
    #[error("Invalid wz version used for decryption, try parsing other version numbers.")]
    InvalidWzVersion,
    #[error("Entry count overflow, Invalid wz version used for decryption, try parsing other version numbers.")]
    InvalidEntryCount,
    #[error("Binary reading error")]
    ReaderError(#[from] reader::Error),
}

#[derive(Debug)]
enum WzDirectoryType {
    UnknownType,
    /// directory type and name maybe at some where alse
    RetrieveStringFromOffset,
    WzDirectory,
    WzImage,
    NewUnknownType,
}

fn get_wz_directory_type_from_byte(byte: u8) -> WzDirectoryType {
    match byte {
        1 => WzDirectoryType::UnknownType,
        2 => WzDirectoryType::RetrieveStringFromOffset,
        3 => WzDirectoryType::WzDirectory,
        4 => WzDirectoryType::WzImage,
        _ => WzDirectoryType::NewUnknownType,
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct WzDirectory {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub reader: Arc<WzReader>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub offset: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub block_size: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub hash: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub is_parsed: bool,
}

impl WzDirectory {
    pub fn new(offset: usize, block_size: usize, reader: &Arc<WzReader>, is_parsed: bool) -> Self {
        Self {
            reader: reader.clone(),
            offset,
            block_size,
            hash: 0,
            is_parsed,
        }
    }
    pub fn with_hash(mut self, hash: usize) -> Self {
        self.hash = hash;
        self
    }

    pub fn verify_hash(&self) -> Result<(), Error> {
        let reader = self.reader.create_slice_reader();

        reader.seek(self.offset);

        let entry_count = reader.read_wz_int()?;

        if !(0..=1000000).contains(&entry_count) {
            return Err(Error::InvalidEntryCount);
        }

        for _ in 0..entry_count {
            let dir_byte = reader.read_u8()?;
            let dir_type = get_wz_directory_type_from_byte(dir_byte);

            match dir_type {
                WzDirectoryType::UnknownType => {
                    reader.skip(4 + 4 + 2);
                    continue;
                }
                WzDirectoryType::RetrieveStringFromOffset => {
                    // skip read string offset
                    reader.skip(4);
                }
                WzDirectoryType::WzDirectory | WzDirectoryType::WzImage => {
                    reader.read_wz_string()?;
                }
                WzDirectoryType::NewUnknownType => {
                    return Err(Error::UnknownWzDirectoryType(dir_byte, reader.pos.get()))
                }
            }

            let fsize = reader.read_wz_int()?;
            reader.read_wz_int()?;
            let offset = reader.read_wz_offset(self.hash, None)?;
            let buf_start = offset;

            let buf_end = buf_start + fsize as usize;

            if !reader.is_valid_pos(buf_end) {
                return Err(Error::InvalidWzVersion);
            }
        }

        Ok(())
    }

    pub fn resolve_children(&self, parent: &WzNodeArc) -> Result<WzNodeArcVec, Error> {
        let reader = self.reader.create_slice_reader();

        reader.seek(self.offset);

        let entry_count = reader.read_wz_int()?;

        if !(0..=1000000).contains(&entry_count) {
            return Err(Error::InvalidEntryCount);
        }

        let mut nodes: WzNodeArcVec = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            let dir_byte = reader.read_u8()?;
            let mut dir_type = get_wz_directory_type_from_byte(dir_byte);

            let fname: WzNodeName;

            match dir_type {
                WzDirectoryType::UnknownType => {
                    /* unknown, just skip this chunk */
                    reader.skip(4 + 4 + 2);
                    continue;
                }
                WzDirectoryType::RetrieveStringFromOffset => {
                    let str_offset = reader.read_i32()?;

                    let offset = reader.header.fstart + str_offset as usize;

                    dir_type = get_wz_directory_type_from_byte(reader.read_u8_at(offset)?);
                    fname = reader.read_wz_string_at_offset(offset + 1)?.into();
                }
                WzDirectoryType::WzDirectory | WzDirectoryType::WzImage => {
                    fname = reader.read_wz_string()?.into();
                }
                WzDirectoryType::NewUnknownType => {
                    println!("NewUnknownType: {}", dir_byte);
                    return Err(Error::UnknownWzDirectoryType(dir_byte, reader.pos.get()));
                }
            }

            let fsize = reader.read_wz_int()?;
            let _checksum = reader.read_wz_int()?;
            let offset = reader.read_wz_offset(self.hash, None)?;
            let buf_start = offset;

            let buf_end = buf_start + fsize as usize;

            if !reader.is_valid_pos(buf_end) {
                return Err(Error::InvalidWzVersion);
            }

            match dir_type {
                WzDirectoryType::WzDirectory => {
                    let wz_dir = WzDirectory::new(offset, fsize as usize, &self.reader, false)
                        .with_hash(self.hash);

                    let obj_node = WzNode::new(&fname, wz_dir, Some(parent));

                    nodes.push((fname, obj_node.into_lock()));
                }
                WzDirectoryType::WzImage => {
                    let wz_image = WzImage::new(&fname, offset, fsize as usize, &self.reader);

                    let obj_node = WzNode::new(&fname, wz_image, Some(parent));

                    nodes.push((fname, obj_node.into_lock()));
                }
                _ => {
                    // should never be here
                }
            }
        }

        for (_, node) in nodes.iter() {
            let mut write = node.write().unwrap();
            if let WzObjectType::Directory(dir) = &mut write.object_type {
                let children = dir.resolve_children(node)?;

                for (name, child) in children {
                    write.children.insert(name, child);
                }
            }
        }

        Ok(nodes)
    }
}
