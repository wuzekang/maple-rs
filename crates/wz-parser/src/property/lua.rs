use crate::util::maple_crypto_constants::{WZ_GMSIV, WZ_MSEAIV};
use crate::util::WzMutableKey;
use crate::WzReader;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WzLuaParseError {
    #[error("Lua decode fail")]
    StringDecodeFail(#[from] std::string::FromUtf8Error),

    #[error("Unknown Lua Iv")]
    UnknownLuaIv,

    #[error("Not a Lua property")]
    NotLuaProperty,
}

/// WzLua use to store lua information and extraction method.
#[derive(Debug, Clone, Default)]
pub struct WzLua {
    reader: Arc<WzReader>,
    offset: usize,
    length: usize,
}

impl WzLua {
    pub fn new(reader: &Arc<WzReader>, offset: usize, length: usize) -> Self {
        Self {
            reader: Arc::clone(reader),
            offset,
            length,
        }
    }

    /// extract lua string from wz file
    pub fn extract_lua(&self) -> Result<String, WzLuaParseError> {
        let data = self
            .reader
            .get_slice(self.offset..self.length + self.offset);

        let mut keys = self
            .get_mtb_keys_from_guess_lua_iv()
            .ok_or(WzLuaParseError::UnknownLuaIv)?;

        let len = data.len();

        let mut decoded = data.to_vec();

        keys.ensure_key_size(len).unwrap();

        keys.decrypt_slice(&mut decoded);

        String::from_utf8(decoded).map_err(WzLuaParseError::from)
    }

    /// try to guess the iv from encrypted data
    fn get_mtb_keys_from_guess_lua_iv(&self) -> Option<WzMutableKey> {
        let len = std::cmp::min(64, self.length);
        let test_data = self.reader.get_slice(self.offset..self.offset + len);

        let ivs = [WZ_MSEAIV, WZ_GMSIV, [0, 0, 0, 0]];

        for iv in ivs {
            let mut decoded = test_data.to_vec();
            let mut keys = WzMutableKey::from_iv(iv);

            keys.ensure_key_size(len).unwrap();

            keys.decrypt_slice(&mut decoded);

            if String::from_utf8(decoded).is_ok() {
                return Some(keys);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::WzReader;

    fn generate_encrypted_text(text: &str, iv: [u8; 4]) -> Vec<u8> {
        let mut keys = WzMutableKey::from_iv(iv);
        let mut data = text.as_bytes().to_vec();

        keys.ensure_key_size(data.len()).unwrap();

        keys.decrypt_slice(&mut data);

        data
    }

    fn setup_lua(iv: [u8; 4]) -> Result<WzLua, std::io::Error> {
        let lua_text = "print(1234567)";
        let len = lua_text.len();
        
        let encrypted = generate_encrypted_text(lua_text, iv);
        
        // 为了测试，创建一个足够大的缓冲区
        let mut buffer = vec![0u8; 1024];
        buffer[..len].copy_from_slice(&encrypted);

        let reader = Arc::new(WzReader::from_bytes(buffer).with_iv(iv));

        Ok(WzLua::new(&reader, 0, len))
    }

    #[test]
    fn should_guess_gms() {
        let lua = setup_lua(WZ_GMSIV).unwrap();

        let text = lua.extract_lua();

        assert!(text.is_ok());
        assert_eq!(text.unwrap(), "print(1234567)");
    }

    #[test]
    fn should_guess_msea() {
        let lua = setup_lua(WZ_MSEAIV).unwrap();

        let text = lua.extract_lua();

        assert!(text.is_ok());
        assert_eq!(text.unwrap(), "print(1234567)");
    }

    #[test]
    fn should_guess_none_iv() {
        let lua = setup_lua([0, 0, 0, 0]).unwrap();

        let text = lua.extract_lua();

        assert!(text.is_ok());
        assert_eq!(text.unwrap(), "print(1234567)");
    }

    #[test]
    fn should_error_unknown_iv() {
        let lua = setup_lua([1, 2, 3, 4]).unwrap();

        let text = lua.extract_lua();

        assert!(text.is_err());
    }
}
