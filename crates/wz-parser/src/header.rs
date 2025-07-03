use super::reader::Error;
use scroll::{ctx, Endian, Pread, LE};

type Result<T> = std::result::Result<T, Error>;

/// Wz file's header.
#[derive(Debug, Clone, Copy, Default)]
pub struct WzHeader<'a> {
    pub ident: &'a str,
    pub fsize: u64,
    /// when wz file's content actually start
    pub fstart: usize,
    pub copyright: &'a str,
}

impl<'a> ctx::TryFromCtx<'a, Endian> for WzHeader<'a> {
    type Error = Error;
    fn try_from_ctx(src: &'a [u8], _: Endian) -> std::result::Result<(Self, usize), Self::Error> {
        Self::read_from_buf(src)
    }
}

impl WzHeader<'_> {
    pub fn get_header_slice(buf: &[u8]) -> &[u8] {
        let fstart = Self::get_wz_fstart(buf).unwrap() as usize;
        &buf[0..fstart]
    }
    pub fn get_ident(buf: &[u8]) -> Result<&str> {
        buf[0..4].pread::<&str>(0).map_err(Error::from)
    }
    pub fn get_wz_fsize(buf: &[u8]) -> Result<u64> {
        buf.pread_with::<u64>(4, LE).map_err(Error::from)
    }
    pub fn get_wz_fstart(buf: &[u8]) -> Result<u32> {
        buf.pread_with::<u32>(12, LE).map_err(Error::from)
    }
    pub fn get_wz_copyright(buf: &[u8]) -> Result<&str> {
        let fstart = Self::get_wz_fstart(buf)? as usize;
        buf[16..fstart - 17].pread::<&str>(0).map_err(Error::from)
    }
    pub fn read_from_buf(buf: &[u8]) -> Result<(WzHeader, usize)> {
        let ident = Self::get_ident(buf)?;

        let fsize = Self::get_wz_fsize(buf)?;

        let fstart = Self::get_wz_fstart(buf)? as usize;

        let copyright = Self::get_wz_copyright(buf)?;

        let offset = fstart - 17;

        Ok((
            WzHeader {
                ident,
                fsize,
                fstart,
                copyright,
            },
            offset,
        ))
    }
}
