use gltf::{buffer::Data, Accessor};
use gltf_json::{
    accessor::{self, ComponentType, GenericComponentType},
    Index,
};
use pp_core::mesh::edge::FlapPosition;
use serde::{Deserialize, Serialize};

use crate::standard::buffers::{self, AccessorOptions, GltfBufferBuilder};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CutPrimitiveAttributes {
    /// The GLTF indices of the two vertices that form the cut edge
    #[serde(rename = "VERTICES")]
    vertices: Index<accessor::Accessor>,
    /// Indicates which face on this edge has the flap (if any)
    #[serde(rename = "FLAP_POSITION")]
    flap_position: Index<accessor::Accessor>,
}

/// Represents the components of a saved cut once assembled
#[derive(Clone, Debug)]
pub struct SerializableCut {
    /// The GLTF indices of the two vertices that form the cut edge
    pub vertices: [u32; 2],
    /// The face index that has the flap (if any)
    pub flap_position: FlapPosition,
}

/// Builds the GLTF accessors that encode "cut"s into a GLTF file's buffers
pub fn save_cuts(
    builder: &mut GltfBufferBuilder,
    cuts: Vec<SerializableCut>,
) -> CutPrimitiveAttributes {
    use gltf_json::validation::Checked::Valid;
    use gltf_json::{accessor, buffer};

    let mut vertices = Vec::with_capacity(cuts.len());
    let mut flap_positions = Vec::<u8>::with_capacity(cuts.len());
    cuts.iter().for_each(|cut| {
        vertices.push(cut.vertices);
        flap_positions.push(cut.flap_position.clone().into());
    });

    CutPrimitiveAttributes {
        vertices: builder.add_accessor(
            &vertices,
            AccessorOptions {
                component_type: Valid(GenericComponentType(ComponentType::U32)),
                type_: Valid(accessor::Type::Vec2),
                target: Some(Valid(buffer::Target::ArrayBuffer)),
                normalized: false,
                min: None,
                max: None,
            },
        ),
        flap_position: builder.add_accessor(
            &flap_positions,
            AccessorOptions {
                component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::U8)),
                type_: Valid(accessor::Type::Scalar),
                target: Some(Valid(buffer::Target::ArrayBuffer)),
                normalized: false,
                min: None,
                max: None,
            },
        ),
    }
}

/// Reads edge "cut"s from a GLTF file's buffers
pub fn load_cuts(
    accessors: &[Accessor],
    buffers: &[Data],
    cuts: CutPrimitiveAttributes,
) -> Vec<SerializableCut> {
    let Some(vertices) = accessors
        .get(cuts.vertices.value())
        .and_then(|accessor| buffers::read_accessor::<[u32; 2]>(buffers, accessor).ok())
    else {
        return Vec::new();
    };

    // Load flap positions if available, otherwise use default
    let flap_positions = accessors
        .get(cuts.flap_position.value())
        .and_then(|accessor| buffers::read_accessor::<u8>(buffers, accessor).ok())
        .map(|positions| positions.into_iter().map(FlapPosition::from).collect())
        .unwrap_or_else(|| vec![FlapPosition::FirstFace; vertices.len()]);

    vertices
        .into_iter()
        .zip(flap_positions)
        .map(|(vertices, flap_position)| SerializableCut { vertices, flap_position })
        .collect()
}
