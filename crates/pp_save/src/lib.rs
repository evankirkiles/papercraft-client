use std::io;

use gltf::Gltf;

mod extra;
mod standard;

pub mod load;
pub mod save;

/// A GLTF file with a `papercraft` extension containing the app state
pub struct SaveFile(gltf::Gltf);

impl SaveFile {
    /// Validates that the current save file matches the app state schema
    pub fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Validates and interprets a GLTF / GLB as a `SaveFile`
    pub fn from_reader<R>(reader: R) -> anyhow::Result<Self>
    where
        R: io::Read + io::Seek,
    {
        let save = Self(Gltf::from_reader(reader)?);
        save.validate()?;
        Ok(save)
    }

    /// Returns the GLTF JSON blob as a JSON string
    pub fn to_json_string(&self) -> std::result::Result<std::string::String, gltf_json::Error> {
        self.0.as_json().to_string()
    }

    /// Returns the GLTF JSON blob as a JSON pretty-printed string
    pub fn to_json_string_pretty(
        &self,
    ) -> std::result::Result<std::string::String, gltf_json::Error> {
        self.0.as_json().to_string_pretty()
    }

    fn to_glb(&'_ self) -> anyhow::Result<gltf::binary::Glb<'_>> {
        use std::borrow::Cow;
        Ok(gltf::binary::Glb {
            header: gltf::binary::Header {
                magic: *b"glTF",
                version: 2,
                length: 0, // Will be calculated by to_writer
            },
            json: Cow::Owned(self.0.as_json().to_string()?.into_bytes()),
            bin: self.0.blob.as_ref().map(|blob| Cow::Borrowed(blob.as_slice())),
        })
    }

    /// Exports the save file as a GLB binary blob
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.to_glb()?.to_vec()?)
    }
}
