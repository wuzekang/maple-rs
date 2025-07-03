use crate::{version::WzMapleVersion, SharedWzMutableKey, WzNode, WzNodeArc, WzNodeCast};
use std::fs::DirEntry;
use std::io;
use std::path::Path;

/// Get a wz file path by directory, like `Map` -> `Map/Map.wz`.
pub fn get_root_wz_file_path(dir: &DirEntry) -> Option<String> {
    let dir_name = dir.file_name();
    let mut inner_wz_name = dir_name.to_str().unwrap().to_string();
    inner_wz_name.push_str(".wz");
    let inner_wz_path = dir.path().join(inner_wz_name);

    if inner_wz_path.try_exists().is_ok() {
        return Some(inner_wz_path.to_str().unwrap().to_string());
    }

    None
}

/// Resolve series of wz files in a directory, and merge *_nnn.wz files into one WzFile.
pub fn resolve_root_wz_file_dir_full(
    dir: impl AsRef<Path>,
    version: Option<WzMapleVersion>,
    patch_version: Option<i32>,
    parent: Option<&WzNodeArc>,
    default_keys: Option<&SharedWzMutableKey>,
) -> Result<WzNodeArc, io::Error> {
    let root_node: WzNodeArc =
        WzNode::from_wz_file_full(&dir, version, patch_version, parent, default_keys)
            .unwrap()
            .into();
    let wz_dir = dir.as_ref().parent().unwrap();

    {
        let mut root_node_write = root_node.write().unwrap();

        root_node_write.parse(&root_node).unwrap();

        for entry in wz_dir.read_dir()? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let name = entry.file_name();

            if file_type.is_dir() && root_node_write.at(name.to_str().unwrap()).is_some() {
                if let Some(file_path) = get_root_wz_file_path(&entry) {
                    let dir_node = resolve_root_wz_file_dir_full(
                        &file_path,
                        version,
                        patch_version,
                        Some(&root_node),
                        default_keys,
                    )?;

                    /* replace the original one */
                    root_node_write
                        .children
                        .insert(name.to_str().unwrap().into(), dir_node);
                }
            } else if file_type.is_file() {
                //  check is XXX_nnn.wz
                let file_path = entry.path();
                let file_name = file_path.file_stem().unwrap().to_str().unwrap();

                let splited = file_name.split('_').collect::<Vec<&str>>();

                if splited.len() < 2 {
                    continue;
                }

                if splited.last().unwrap().parse::<u16>().is_err() {
                    continue;
                }

                let node = WzNode::from_wz_file_full(
                    file_path.to_str().unwrap(),
                    version,
                    patch_version,
                    None,
                    default_keys,
                )
                .unwrap()
                .into_lock();

                let mut node_write = node.write().unwrap();

                if node_write.parse(&root_node).is_ok() {
                    root_node_write.children.reserve(node_write.children.len());
                    for (name, child) in node_write.children.drain(..) {
                        root_node_write.children.insert(name, child);
                    }
                }
            }
        }
    }

    Ok(root_node)
}

/// resolve_root_wz_file_dir_full with less arguments for easier use
pub fn resolve_root_wz_file_dir(
    dir: impl AsRef<Path>,
    parent: Option<&WzNodeArc>,
) -> Result<WzNodeArc, io::Error> {
    resolve_root_wz_file_dir_full(dir, None, None, parent, None)
}

/// Construct `WzNode` tree from `Base.wz`
pub fn resolve_base(
    path: impl AsRef<Path>,
    version: Option<WzMapleVersion>,
) -> Result<WzNodeArc, io::Error> {
    if !path.as_ref().ends_with("Base.wz") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "not a Base.wz"));
    }

    let base_node = resolve_root_wz_file_dir_full(&path, version, None, None, None)?;

    let (patch_version, keys) = {
        let node_read = base_node.read().unwrap();
        let file = node_read.try_as_file().unwrap();

        // reusing the keys from Base.wz
        (file.wz_file_meta.patch_version, file.reader.keys.clone())
    };

    {
        let mut base_write = base_node.write().unwrap();

        let first_parent = path.as_ref().parent().unwrap();

        // if a Base.wz in under Base folder, then should up to parent to find Map, Item and other stuff
        // if not, the other stuff just at same folder as Base.wz
        let wz_root_path = if first_parent.file_stem().unwrap() == "Base" {
            first_parent.parent().unwrap()
        } else {
            first_parent
        };

        for item in wz_root_path.read_dir()? {
            let dir = item?;
            let path = dir.path();
            let file_name = path.file_stem().unwrap();

            let is_valid = path.extension().unwrap_or_default() == "wz" || path.is_dir();

            // we only allow the thing is listed in Base.wz
            let has_dir = base_write.at(file_name.to_str().unwrap()).is_some();

            if has_dir && is_valid {
                let wz_path = if dir.file_type()?.is_dir() {
                    get_root_wz_file_path(&dir)
                } else {
                    Some(path.to_str().unwrap().to_string())
                };

                if let Some(file_path) = wz_path {
                    let dir_node = resolve_root_wz_file_dir_full(
                        &file_path,
                        version,
                        Some(patch_version),
                        Some(&base_node),
                        Some(&keys),
                        // None,
                    )?;

                    /* replace the original one */
                    base_write
                        .children
                        .insert(file_name.to_str().unwrap().into(), dir_node);
                }
            }
        }
    }

    Ok(base_node)
}
