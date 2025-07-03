use crate::util::maple_crypto_constants::{WZ_GMSIV, WZ_MSEAIV};
use crate::util::wz_mutable_key::WzMutableKey;
use crate::{reader, Reader, WzHeader, WzSliceReader};
use std::sync::{Arc, RwLock};

pub fn get_iv_by_maple_version(version: WzMapleVersion) -> [u8; 4] {
    match version {
        WzMapleVersion::GMS => WZ_GMSIV,
        WzMapleVersion::EMS => WZ_MSEAIV,
        _ => [0; 4],
    }
}

/// MapleStory version, use to determine the IV for decryption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WzMapleVersion {
    /// Global MapleStory (old)
    GMS,

    /// 新楓之谷 / 冒险岛Online / 메이플스토리 / MapleSEA / EMS (old)
    EMS,

    /// BMS / GMS / MapleSEA / メイプルストーリー / 메이플스토리
    BMS,

    CLASSIC,

    GENERATE,

    /* from zlz.dll */
    GETFROMZLZ,

    CUSTOM,

    UNKNOWN,
}

/// Verify IV from wz image
pub fn verify_iv_from_wz_img(buf: &[u8], iv: &[u8; 4]) -> bool {
    let reader = WzSliceReader::new(buf, &Arc::new(RwLock::new(WzMutableKey::from_iv(*iv))));

    reader.pos.set(1);

    reader.read_wz_string().unwrap_or_default() == "Property"
}

/// Try to guess IV from wz image use fixed value. Currently will try GMS, EMS, BMS.
pub fn guess_iv_from_wz_img(buf: &[u8]) -> Option<[u8; 4]> {
    // not support other then WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET
    if buf[0] != 0x73 {
        return None;
    }

    let guess_versions = [
        WzMapleVersion::GMS,
        WzMapleVersion::EMS,
        WzMapleVersion::BMS,
    ];

    for version in guess_versions.iter() {
        let iv = get_iv_by_maple_version(*version);
        if verify_iv_from_wz_img(buf, &iv) {
            return Some(iv);
        }
    }

    None
}

pub fn verify_iv_from_wz_file(buf: &[u8], iv: &[u8; 4]) -> Result<(), reader::Error> {
    let reader = WzSliceReader::new(buf, &Arc::new(RwLock::new(WzMutableKey::from_iv(*iv))));

    let fstart = WzHeader::get_wz_fstart(buf)? as usize;

    reader.seek(fstart + 2);

    let entry_count = reader.read_wz_int()?;

    if !(0..=1000000).contains(&entry_count) {
        return Err(reader::Error::DecryptError(reader.pos.get()));
    }

    for _ in 0..entry_count {
        let dir_byte = reader.read_u8()?;

        match dir_byte {
            1 => {
                reader.skip(4 + 4 + 2);
                continue;
            }
            2 => {
                let str_offset = reader.read_i32()?;

                let offset = reader.header.fstart + str_offset as usize;
                // just check string can be valid string(instead of parse string lossy), so can prove the iv is valid
                let meta = reader.read_wz_string_meta_at(offset + 1)?;
                reader.try_resolve_wz_string_meta(
                    &meta.string_type,
                    meta.offset,
                    meta.length as usize,
                )?;
            }
            3 | 4 => {
                // just check string can be valid string(instead of parse string lossy), so can prove the iv is valid
                let meta = reader.read_wz_string_meta()?;
                reader.try_resolve_wz_string_meta(
                    &meta.string_type,
                    meta.offset,
                    meta.length as usize,
                )?;
            }
            _ => return Err(reader::Error::DecryptError(reader.pos.get())),
        }

        reader.read_wz_int()?;
        reader.read_wz_int()?;
        reader.skip(4);
    }

    Ok(())
}

pub fn guess_iv_from_wz_file(buf: &[u8]) -> Option<[u8; 4]> {
    let guess_versions = [
        WzMapleVersion::BMS,
        WzMapleVersion::GMS,
        WzMapleVersion::EMS,
    ];

    for version in guess_versions.iter() {
        let iv = get_iv_by_maple_version(*version);
        if verify_iv_from_wz_file(buf, &iv).is_ok() {
            return Some(iv);
        }
    }

    None
}
