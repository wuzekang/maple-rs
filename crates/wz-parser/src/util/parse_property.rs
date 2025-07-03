use crate::property::{
    get_sound_type_from_header, Vector2D, WzPng, WzRawData, WzSound, WzString, WzSubProperty,
    WzValue,
};
use crate::{
    reader, WzNode, WzNodeArc, WzNodeArcVec, WzNodeName, WzObjectType, WzReader, WzSliceReader,
};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WzPropertyParseError {
    #[error("Node not found")]
    NodeNotFound,

    #[error("Unknown property type: {0}, at position: {1}")]
    UnknownPropertyType(u8, usize),

    #[error("Unknown extended header type: {0}, at position: {1}")]
    UnknownExtendedHeaderType(u8, usize),

    #[error("Unknown extended property type: {0}, at position: {1}")]
    UnknownExtendedPropertyType(String, usize),

    #[error("Binary reading error")]
    ReaderError(#[from] reader::Error),
}

pub fn parse_property_list(
    parent: Option<&WzNodeArc>,
    org_reader: &Arc<WzReader>,
    reader: &WzSliceReader,
    origin_offset: usize,
) -> Result<(WzNodeArcVec, Vec<WzNodeArc>), WzPropertyParseError> {
    let entry_count = reader.read_wz_int()?;

    let mut childs: WzNodeArcVec = Vec::with_capacity(entry_count as usize);
    let mut uol_nodes: Vec<WzNodeArc> = Vec::new();

    for _ in 0..entry_count {
        let name: WzNodeName = reader.read_wz_string_block(origin_offset)?.into();
        let property_type = reader.read_u8()?;
        let parsed_node = parse_property_node(
            name,
            property_type,
            parent,
            org_reader,
            reader,
            origin_offset,
        )?;

        if let Some(uol_node) = parsed_node.2 {
            uol_nodes.extend(uol_node);
        }

        childs.push((parsed_node.0, parsed_node.1));
    }

    Ok((childs, uol_nodes))
}

pub fn parse_property_node(
    name: WzNodeName,
    property_type: u8,
    parent: Option<&WzNodeArc>,
    org_reader: &Arc<WzReader>,
    reader: &WzSliceReader,
    origin_offset: usize,
) -> Result<(WzNodeName, WzNodeArc, Option<Vec<WzNodeArc>>), WzPropertyParseError> {
    let result: (WzNodeName, WzNodeArc);

    match property_type {
        0 => {
            let node = WzNode::new(&name, WzObjectType::Value(WzValue::Null), parent);
            result = (name, node.into_lock());
        }
        2 | 11 => {
            let num = reader.read_i16()?;
            let node = WzNode::new(&name, num, parent);
            result = (name, node.into_lock());
        }
        3 | 19 => {
            let num = reader.read_wz_int()?;
            let node = WzNode::new(&name, num, parent);
            result = (name, node.into_lock());
        }
        20 => {
            let num = reader.read_wz_int64()?;
            let node = WzNode::new(&name, num, parent);
            result = (name, node.into_lock());
        }
        4 => {
            let float_type: u8 = reader.read_u8()?;
            match float_type {
                0x80 => {
                    let num = reader.read_float()?;
                    let node = WzNode::new(&name, num, parent);
                    result = (name, node.into_lock());
                }
                _ => {
                    let node = WzNode::new(&name, float_type as f32, parent);
                    result = (name, node.into_lock());
                }
            }
        }
        5 => {
            let num = reader.read_double()?;
            let node = WzNode::new(&name, num, parent);
            result = (name, node.into_lock());
        }
        8 => {
            let str_meta = reader.read_wz_string_block_meta(origin_offset)?;
            let node = WzNode::new(&name, WzString::from_meta(str_meta, org_reader), parent);
            result = (name, node.into_lock());
        }
        9 => {
            let block_size = reader.read_u32()?;
            let next_pos = reader.pos.get() + block_size as usize;

            let node =
                parse_extended_prop(parent, org_reader, reader, next_pos, origin_offset, name)?;

            reader.seek(next_pos);

            return Ok(node);
        }
        _ => {
            return Err(WzPropertyParseError::UnknownPropertyType(
                property_type,
                reader.pos.get(),
            ));
        }
    }
    Ok((result.0, result.1, None))
}

pub fn parse_extended_prop(
    parent: Option<&WzNodeArc>,
    org_reader: &Arc<WzReader>,
    reader: &WzSliceReader,
    end_of_block: usize,
    origin_offset: usize,
    property_name: WzNodeName,
) -> Result<(WzNodeName, WzNodeArc, Option<Vec<WzNodeArc>>), WzPropertyParseError> {
    let extend_property_type = reader.read_wz_string_block(origin_offset)?;
    parse_more(
        parent,
        org_reader,
        reader,
        end_of_block,
        origin_offset,
        property_name,
        &extend_property_type,
    )
}

pub fn parse_more(
    parent: Option<&WzNodeArc>,
    org_reader: &Arc<WzReader>,
    reader: &WzSliceReader,
    end_of_block: usize,
    origin_offset: usize,
    property_name: WzNodeName,
    extend_property_type: &str,
) -> Result<(WzNodeName, WzNodeArc, Option<Vec<WzNodeArc>>), WzPropertyParseError> {
    match extend_property_type {
        "Property" => {
            let node = WzNode::new(
                &property_name,
                WzObjectType::Property(WzSubProperty::Property),
                parent,
            )
            .into_lock();

            reader.skip(2);
            let (childs, uol_nodes) =
                parse_property_list(Some(&node), org_reader, reader, origin_offset)?;

            {
                let mut node_write = node.write().unwrap();
                node_write.children.reserve(childs.len());
                for (name, child) in childs {
                    node_write.children.insert(name, child);
                }
            }

            Ok((property_name, node, Some(uol_nodes)))
        }
        "Canvas" => {
            reader.skip(1);
            let has_child = reader.read_u8()? == 1;

            let node = WzNode::new(
                &property_name,
                WzObjectType::Property(WzSubProperty::Property),
                parent,
            )
            .into_lock();

            let mut uol_nodes: Option<Vec<WzNodeArc>> = None;

            if has_child {
                reader.skip(2);
                let (childs, uols) =
                    parse_property_list(Some(&node), org_reader, reader, origin_offset)?;
                let mut node_write = node.write().unwrap();
                node_write.children.reserve(childs.len());
                for (name, child) in childs {
                    node_write.children.insert(name, child);
                }
                uol_nodes = Some(uols);
            }

            let width = reader.read_wz_int()?;
            let height = reader.read_wz_int()?;
            let format1 = reader.read_wz_int()?;
            let format2 = reader.read_i8()?;
            reader.skip(4);
            let canvas_slice_size = (reader.read_i32()? - 1) as usize;
            reader.skip(1);
            let canvas_offset = reader.pos.get();
            let canvas_header = reader.read_u16()?;
            let wz_png = WzPng::new(
                org_reader,
                (width as u32, height as u32),
                (format1 as u32, format2 as u32),
                (canvas_offset, canvas_slice_size),
                canvas_header as i32,
            );

            if let Ok(mut node) = node.write() {
                node.object_type = wz_png.into();
            }

            Ok((property_name, node, uol_nodes))
        }
        "Shape2D#Convex2D" => {
            let node = WzNode::new(
                &property_name,
                WzObjectType::Property(WzSubProperty::Convex),
                parent,
            )
            .into_lock();

            let entry_count = reader.read_wz_int()?;
            let mut uol_nodes: Vec<WzNodeArc> = Vec::new();

            {
                let mut node_write = node.write().unwrap();
                node_write.children.reserve(entry_count as usize);
                for i in 0..entry_count {
                    let name: WzNodeName = i.to_string().into();
                    let parsed_node = parse_extended_prop(
                        Some(&node),
                        org_reader,
                        reader,
                        end_of_block,
                        origin_offset,
                        name,
                    )?;

                    if let Some(uols) = parsed_node.2 {
                        uol_nodes.extend(uols);
                    }

                    node_write.children.insert(parsed_node.0, parsed_node.1);
                }
            }

            Ok((property_name, node, Some(uol_nodes)))
        }
        "Shape2D#Vector2D" => {
            let vec2 = Vector2D(reader.read_wz_int()?, reader.read_wz_int()?);
            let node = WzNode::new(&property_name, vec2, parent);

            Ok((property_name, node.into_lock(), None))
        }
        "Sound_DX8" => {
            reader.skip(1);
            let _sound_start_offset = reader.pos.get();
            let sound_size = reader.read_wz_int()? as u32;
            let sound_duration = reader.read_wz_int()? as u32;
            let sound_offset = end_of_block - (sound_size as usize);

            let header_offset: usize = reader.pos.get();

            let header_size = sound_offset - header_offset;

            let sound_type = get_sound_type_from_header(
                &reader.buf[header_offset..header_offset + header_size],
                sound_size,
                sound_duration,
            );
            let sound = WzSound::new(
                org_reader,
                sound_offset,
                sound_size,
                header_offset,
                header_size,
                sound_duration,
                sound_type,
            );

            let node = WzNode::new(&property_name, sound, parent);

            Ok((property_name, node.into_lock(), None))
        }
        "UOL" => {
            reader.skip(1);
            let str_meta = reader.read_wz_string_block_meta(origin_offset)?;
            let node = WzNode::new(
                &property_name,
                WzObjectType::Value(WzValue::UOL(WzString::from_meta(str_meta, org_reader))),
                parent,
            )
            .into_lock();

            let uol_nodes = Some(vec![Arc::clone(&node)]);

            Ok((property_name, node, uol_nodes))
        }
        "RawData" => {
            reader.skip(1);
            let raw_data_size = reader.read_i32()? as usize;
            let raw_data_offset = reader.pos.get();
            let node = WzNode::new(
                &property_name,
                WzRawData::new(org_reader, raw_data_offset, raw_data_size),
                parent,
            );

            Ok((property_name, node.into_lock(), None))
        }
        _ => Err(WzPropertyParseError::UnknownExtendedPropertyType(
            extend_property_type.to_string(),
            reader.pos.get(),
        )),
    }
}

/// Direct get node from path with providing reader. see [`crate::WzImage::at_path`].
pub fn get_node(
    path: &str,
    org_reader: &Arc<WzReader>,
    reader: &WzSliceReader,
    origin_offset: usize,
) -> Result<(WzNodeName, WzNodeArc), WzPropertyParseError> {
    if path.is_empty() {
        return Err(WzPropertyParseError::NodeNotFound);
    }

    let mut pathes = path.split('/');
    let mut current_path = pathes.next();

    while let Some(current_name) = current_path {
        let entry_count = reader.read_wz_int()?;
        let next_path = pathes.next();
        for _ in 0..entry_count {
            let name = reader.read_wz_string_block(origin_offset)?;
            let property_type = reader.read_u8()?;

            if name == current_name && next_path.is_none() {
                let result = parse_property_node(
                    name.into(),
                    property_type,
                    None,
                    org_reader,
                    reader,
                    origin_offset,
                )?;
                return Ok((result.0, result.1));
            }

            match property_type {
                0 => { /* do nothing */ }
                2 | 11 => {
                    reader.skip(2);
                }
                3 | 19 => {
                    reader.read_wz_int()?;
                }
                20 => {
                    reader.read_wz_int64()?;
                }
                4 => {
                    let float_type: u8 = reader.read_u8()?;

                    if float_type == 0x80 {
                        reader.skip(4);
                    }
                }
                5 => {
                    reader.skip(8);
                }
                8 => {
                    let string_type = reader.read_u8()?;

                    match string_type {
                        0 | 0x73 => {
                            reader.read_wz_string_meta()?;
                        }
                        1 | 0x1B => {
                            reader.skip(4);
                        }
                        _ => {}
                    }
                }
                9 => {
                    if name == current_name {
                        current_path = next_path;
                        // skip block size
                        reader.skip(4);
                        reader.read_wz_string_block_meta(origin_offset)?;
                        reader.skip(2);
                        break;
                    } else {
                        let block_size = reader.read_u32()?;
                        reader.skip(block_size as usize);
                    }
                }
                _ => {
                    return Err(WzPropertyParseError::UnknownPropertyType(
                        property_type,
                        reader.pos.get(),
                    ));
                }
            }
        }

        if next_path.is_none() {
            break;
        }
    }

    Err(WzPropertyParseError::NodeNotFound)
}
