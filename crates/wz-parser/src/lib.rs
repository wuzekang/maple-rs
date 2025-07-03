pub mod directory;
pub mod file;
mod header;
pub mod node;
mod node_cast;
mod node_name;
mod object;
pub mod property;
pub mod reader;
pub mod util;
pub mod version;
pub mod wz_image;

pub use directory::WzDirectory;
pub use file::WzFile;
pub use header::*;
pub use node::{WzNode, WzNodeArc, WzNodeArcVec};
pub use node_cast::*;
pub use node_name::*;
pub use object::*;
pub use reader::{Reader, SharedWzMutableKey, WzBytesReader, WzSliceReader};

// Alias for backward compatibility
pub type WzReader = WzBytesReader;
pub use wz_image::{
    WzImage, WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET, WZ_IMAGE_HEADER_BYTE_WITH_OFFSET,
};