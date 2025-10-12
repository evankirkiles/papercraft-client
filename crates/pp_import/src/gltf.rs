use gltf::Gltf;
use std::io::Cursor;
use thiserror::Error;

use pp_save::{load::Loadable, save::Saveable, SaveFile};

#[derive(Debug, Clone, Copy, Error)]
pub enum ImportGltfError {
    #[error("unknown GLTF import error")]
    Unknown,
}

impl From<gltf::Error> for ImportGltfError {
    fn from(value: gltf::Error) -> Self {
        ImportGltfError::Unknown
    }
}

impl From<pp_save::load::LoadError> for ImportGltfError {
    fn from(value: pp_save::load::LoadError) -> Self {
        ImportGltfError::Unknown
    }
}

impl From<pp_save::save::SaveError> for ImportGltfError {
    fn from(value: pp_save::save::SaveError) -> Self {
        ImportGltfError::Unknown
    }
}

pub trait GlbSupport {
    fn from_glb(bytes: &'static [u8]) -> Result<SaveFile, ImportGltfError>;
    fn import_glb(save: &mut SaveFile, bytes: &'static [u8; 0]);
}

impl GlbSupport for pp_save::SaveFile {
    fn from_glb(bytes: &'static [u8]) -> Result<SaveFile, ImportGltfError> {
        let cursor = Cursor::new(bytes);
        SaveFile::from_reader(cursor).map_err(|_| ImportGltfError::Unknown)
    }

    fn import_glb(save: &mut SaveFile, bytes: &'static [u8; 0]) {
        todo!()
    }
}
