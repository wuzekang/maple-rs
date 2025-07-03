use super::maple_crypto_constants::{get_trimmed_user_key, MAPLESTORY_USERKEY_DEFAULT, WZ_MSEAIV};
use crate::reader::read_i32_at;
use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, BlockSizeUser, KeyInit};
use aes::Aes256;

const BATCH_SIZE: f64 = 4096_f64;

/// A string decryption util.
#[derive(Debug)]
pub struct WzMutableKey {
    pub iv: [u8; 4],
    keys: Vec<u8>,
    aes_key: [u8; 32],
    /// iv == 0, without decrypt
    pub without_decrypt: bool,
}

impl WzMutableKey {
    pub fn new(iv: [u8; 4], aes_key: [u8; 32]) -> Self {
        Self {
            iv,
            keys: vec![],
            aes_key,
            without_decrypt: read_i32_at(&iv, 0).unwrap_or(0) == 0,
        }
    }
    pub fn new_lua() -> Self {
        Self {
            iv: WZ_MSEAIV,
            keys: vec![],
            aes_key: get_trimmed_user_key(&MAPLESTORY_USERKEY_DEFAULT),
            without_decrypt: false,
        }
    }
    pub fn from_iv(iv: [u8; 4]) -> Self {
        Self {
            iv,
            keys: vec![],
            aes_key: get_trimmed_user_key(&MAPLESTORY_USERKEY_DEFAULT),
            without_decrypt: read_i32_at(&iv, 0).unwrap_or(0) == 0,
        }
    }
    /// force get key at index, will expand key size if not enough.
    pub fn at(&mut self, index: usize) -> &u8 {
        if self.keys.len() <= index {
            self.ensure_key_size(index + 1).unwrap();
        }
        &self.keys[index]
    }
    /// get key at index, return `None` if doesn't exist.
    pub fn try_at(&self, index: usize) -> Option<&u8> {
        self.keys.get(index)
    }
    pub fn get_range(&self, range: std::ops::Range<usize>) -> &[u8] {
        &self.keys[range]
    }
    pub fn is_enough(&self, size: usize) -> bool {
        self.keys.len() >= size
    }
    /// decrypt data in place, make sure has enough key size.
    pub fn decrypt_slice(&self, data: &mut [u8]) {
        if self.without_decrypt {
            return;
        }
        let keys = &self.keys[0..data.len()];
        data.iter_mut()
            .zip(keys)
            .for_each(|(byte, key)| *byte ^= key);
    }
    /// ensure keys has enough size to do decryption
    pub fn ensure_key_size(&mut self, size: usize) -> Result<(), String> {
        if self.is_enough(size) || self.without_decrypt {
            return Ok(());
        }

        let size = (((size as f64) / BATCH_SIZE).ceil() * BATCH_SIZE) as usize;

        if self.keys.capacity() < size {
            self.keys.reserve(size - self.keys.capacity());
        }

        // initialize the first block
        if self.keys.is_empty() {
            self.keys.resize(32, 0);

            let mut block = [0_u8; 16];
            for (index, item) in block.iter_mut().enumerate() {
                *item = self.iv[index % 4];
            }
            ecb::Encryptor::<Aes256>::new(&self.aes_key.into())
                .encrypt_padded_b2b_mut::<Pkcs7>(&block, &mut self.keys)
                .map_err(|_| "Failed to encrypt block")?;

            self.keys.truncate(16);
        }

        let start_index = self.keys.len();

        // fill enouth 0 for later,
        if self.keys.len() < size {
            // + 16 is prevent encryption not enough to padding
            self.keys.resize(size + 16, 0);
        }

        let block_size = aes::Aes256::block_size();

        for i in (start_index..size).step_by(16) {
            let in_buf = self.keys[i - block_size..i].to_vec();
            ecb::Encryptor::<Aes256>::new(&self.aes_key.into())
                // im not sure why this will actually append block_size * 2 to out_buff, so will be trimed at the end
                .encrypt_padded_b2b_mut::<Pkcs7>(&in_buf, &mut self.keys[i..])
                .map_err(|_| "Failed to encrypt block")?;
        }

        if self.keys.len() > size {
            self.keys.truncate(size);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_expand_key() {
        let mut key = WzMutableKey::new_lua();

        assert!(key.ensure_key_size(16).is_ok());
        assert_eq!(key.keys.len(), 4096);

        assert!(key.ensure_key_size(4200).is_ok());
        assert_eq!(key.keys.len(), 4096 * 2);

        assert!(key.ensure_key_size(4096 * 4 + 5).is_ok());
        assert_eq!(key.keys.len(), 4096 * 5);
    }

    #[test]
    fn test_force_at() {
        let mut key = WzMutableKey::new_lua();

        let _ = key.at(1);

        assert_eq!(key.keys.len(), 4096);

        let _ = key.at(4000);

        assert_eq!(key.keys.len(), 4096);

        let _ = key.at(4097);

        assert_eq!(key.keys.len(), 4096 * 2);
    }

    #[test]
    fn test_at() {
        let mut key = WzMutableKey::new_lua();

        assert!(key.try_at(1).is_none());
        assert!(key.try_at(100).is_none());
        assert!(key.try_at(10000).is_none());

        assert!(key.ensure_key_size(10000).is_ok());

        assert!(key.try_at(1).is_some());
        assert!(key.try_at(100).is_some());
        assert!(key.try_at(10000).is_some());
        assert!(key.try_at(20000).is_none());
    }
}
