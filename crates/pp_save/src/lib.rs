mod extra;
mod gltf;

pub mod load;
pub mod save;

/// A GLTF file with a `papercraft` extension containing the app state
pub struct SaveFile(gltf_json::Root);

impl SaveFile {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.0)
    }
}
