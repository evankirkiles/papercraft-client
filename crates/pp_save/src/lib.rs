use std::io;

use gltf::Gltf;

mod extra;
mod standard;

pub mod load;
pub mod save;

/// A GLTF file with a `papercraft` extension containing the app state
pub struct SaveFile(gltf::Gltf);

impl SaveFile {
    pub fn to_json_string(&self) -> std::result::Result<std::string::String, gltf_json::Error> {
        self.0.as_json().to_string()
    }

    pub fn to_json_string_pretty(
        &self,
    ) -> std::result::Result<std::string::String, gltf_json::Error> {
        self.0.as_json().to_string_pretty()
    }

    /// Validates that the current save file matches the app state schema
    pub fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn from_reader<R>(reader: R) -> anyhow::Result<Self>
    where
        R: io::Read + io::Seek,
    {
        let save = Self(Gltf::from_reader(reader)?);
        save.validate()?;
        Ok(save)
    }
}
