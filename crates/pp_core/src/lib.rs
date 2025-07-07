use mesh::MaterialSlotId;
use slotmap::{new_key_type, SecondaryMap, SlotMap};

pub mod commands;
pub mod cut;
pub mod id;
pub mod material;
pub mod measures;
pub mod mesh;
pub mod print;
pub mod select;
pub mod settings;

pub use commands::*;

new_key_type! {
    pub struct MeshId;
    pub struct MaterialId;
    pub struct TextureId;
    pub struct ImageId;
    pub struct SamplerId;
    pub struct PageId;
    pub struct TextBoxId;
    pub struct ImageBoxId;
}

/// IDs of default rendering items you can use for a given mesh
#[derive(Debug)]
pub struct StateDefaults {
    pub material: MaterialId,
    pub texture: TextureId,
    pub sampler: SamplerId,
    pub image: ImageId,
}

/// Represents the entire state of the "core" editor. Note that this closely
/// mimics the structure of a GLTF file.
#[derive(Debug)]
pub struct State {
    // Geometry-related entities
    pub meshes: SlotMap<MeshId, mesh::Mesh>,
    pub materials: SlotMap<MaterialId, material::Material>,
    pub textures: SlotMap<TextureId, material::texture::Texture>,
    pub samplers: SlotMap<SamplerId, material::texture::Sampler>,
    pub images: SlotMap<ImageId, material::image::Image>,

    // Print-related entities
    pub printing: print::PrintLayout,
    pub text_boxes: SlotMap<TextBoxId, print::TextBox>,
    pub image_boxes: SlotMap<ImageBoxId, print::ImageBox>,

    /// Default entity IDs to use where providing an entity is optional
    pub defaults: StateDefaults,

    /// A map from mesh material "slot"s to the actual materials used by them
    pub mesh_materials: SecondaryMap<MeshId, SecondaryMap<MaterialSlotId, MaterialId>>,
    pub selection: select::SelectionState,
    pub settings: settings::Settings,
}

impl Default for State {
    fn default() -> Self {
        // Default image is a 1x1 white pixel
        let mut images: SlotMap<ImageId, material::image::Image> = SlotMap::with_key();
        let image = images.insert(Default::default());

        // Default sampler wraps on U V axes
        let mut samplers: SlotMap<SamplerId, material::texture::Sampler> = SlotMap::with_key();
        let sampler = samplers.insert(Default::default());

        // Default texture is just a combination of default image + sampler
        let mut textures: SlotMap<TextureId, material::texture::Texture> = SlotMap::with_key();
        let texture = textures.insert(material::texture::Texture {
            label: "default".to_string(),
            image,
            sampler,
        });

        // Default material uses default texture as its base
        let mut materials: SlotMap<MaterialId, material::Material> = SlotMap::with_key();
        let material = materials.insert(material::Material {
            label: "default".to_string(),
            base_color_texture: texture,
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            is_dirty: true,
        });

        Self {
            materials,
            textures,
            samplers,
            images,
            meshes: Default::default(),
            printing: Default::default(),
            text_boxes: Default::default(),
            image_boxes: Default::default(),
            selection: Default::default(),
            settings: Default::default(),
            mesh_materials: Default::default(),
            defaults: StateDefaults { material, texture, sampler, image },
        }
    }
}

impl State {
    pub fn add_mesh(
        &mut self,
        mesh: mesh::Mesh,
        materials: Option<SecondaryMap<MaterialSlotId, MaterialId>>,
    ) {
        let m_id = self.meshes.insert(mesh);
        self.mesh_materials.insert(m_id, materials.unwrap_or_default());
    }

    pub fn with_cube() -> Self {
        let mut state = Self::default();
        state.add_mesh(mesh::Mesh::new_cube(), None);
        state
    }
}
