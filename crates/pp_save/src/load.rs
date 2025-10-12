use crate::{gltf, SaveFile};

/// Possible errors that can occur while saving a file
#[derive(Debug, Clone, Copy)]
pub enum LoadError {
    Unknown,
}

pub trait Loadable {
    fn load(save: SaveFile) -> Result<pp_core::State, LoadError>;
}

impl Loadable for pp_core::State {
    fn load(save: SaveFile) -> Result<pp_core::State, LoadError> {
        todo!();
    }
}
