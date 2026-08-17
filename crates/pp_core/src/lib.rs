use slotmap::{new_key_type, SlotMap};

pub mod bounds;
pub mod commands;
pub mod id;
pub mod material;
pub mod measures;
pub mod mesh;
pub mod print;
pub mod select;

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
    pub textures: SlotMap<TextureId, material::Texture>,
    pub samplers: SlotMap<SamplerId, material::Sampler>,
    pub images: SlotMap<ImageId, material::Image>,

    // Print-related entities
    pub printing: print::PrintLayout,
    pub text_boxes: SlotMap<TextBoxId, print::TextBox>,
    pub image_boxes: SlotMap<ImageBoxId, print::ImageBox>,

    /// Default entity IDs to use where providing a link is optional (materials, textures, etc.)
    pub defaults: StateDefaults,

    /// User-specific selection state, to be moved out of this struct
    pub selection: select::SelectionState,
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
            defaults: StateDefaults { material, texture, sampler, image },
        }
    }
}

impl State {
    pub fn with_cube() -> Self {
        let mut state = Self::default();
        state.meshes.insert(mesh::Mesh::new_cube());
        state
    }

    /// The world-space bounding box of every mesh in the document, or
    /// `Aabb3::EMPTY` if there are no meshes / vertices.
    pub fn world_bounds(&self) -> bounds::Aabb3 {
        self.meshes.values().fold(bounds::Aabb3::EMPTY, |acc, m| acc.union(&m.world_aabb()))
    }

    /// Grows or shrinks the printable page grid so that every piece in the
    /// printable quadrant sits over a page. Cheap to call every frame: it only
    /// marks the layout dirty when the number of pages actually changes.
    pub fn fit_pages_to_pieces(&mut self) {
        let bounds = self.piece_bounds_in_print_quadrant();
        self.printing.fit_to_bounds(&bounds);
    }
}
