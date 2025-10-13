use gltf::{buffer::Data, Accessor};
use gltf_json::{
    accessor::{self, ComponentType, GenericComponentType},
    Index,
};
use serde::{Deserialize, Serialize};

use crate::standard::buffers::{self, AccessorOptions, GltfBufferBuilder};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PiecePrimitiveAttributes {
    /// Root face index for each piece
    #[serde(rename = "FACE")]
    pub face: Index<accessor::Accessor>,
    /// Transformation matrices for each piece (column-major, 16 floats per piece)
    #[serde(rename = "TRANSFORM")]
    pub transform: Index<accessor::Accessor>,
}

/// Represents the components of a saved piece once assembled
#[derive(Clone, Debug)]
pub struct SerializablePiece {
    /// Root face index
    pub face_index: u32,
    /// 4x4 transformation matrix (column-major order)
    pub transform: cgmath::Matrix4<f32>,
}

/// Builds the GLTF accessors that encode "piece"s into a GLTF file's buffers
pub fn save_pieces(
    builder: &mut GltfBufferBuilder,
    pieces: Vec<SerializablePiece>,
) -> PiecePrimitiveAttributes {
    use gltf_json::validation::Checked::Valid;
    use gltf_json::{accessor, buffer};

    // Decompose all of the mesh's pieces into serializable components
    let mut face_indices = Vec::<u32>::with_capacity(pieces.len());
    let mut transforms = Vec::<[[f32; 4]; 4]>::with_capacity(pieces.len());
    pieces.iter().for_each(|piece| {
        face_indices.push(piece.face_index);
        transforms.push(piece.transform.into());
    });

    PiecePrimitiveAttributes {
        face: builder.add_accessor(
            &face_indices,
            AccessorOptions {
                component_type: Valid(GenericComponentType(ComponentType::U32)),
                type_: Valid(accessor::Type::Scalar),
                target: Some(Valid(buffer::Target::ArrayBuffer)),
                normalized: false,
                min: None,
                max: None,
            },
        ),
        transform: builder.add_accessor(
            &transforms,
            AccessorOptions {
                component_type: Valid(GenericComponentType(ComponentType::F32)),
                type_: Valid(accessor::Type::Mat4),
                target: Some(Valid(buffer::Target::ArrayBuffer)),
                normalized: false,
                min: None,
                max: None,
            },
        ),
    }
}

/// Reads piece data from a GLTF file's buffers
pub fn load_pieces(
    accessors: &[Accessor],
    buffers: &[Data],
    pieces: PiecePrimitiveAttributes,
) -> Vec<SerializablePiece> {
    let Some(face_indices) = accessors
        .get(pieces.face.value())
        .and_then(|accessor| buffers::read_accessor::<u32>(buffers, accessor).ok())
    else {
        return Vec::new();
    };

    let Some(transforms) = accessors
        .get(pieces.transform.value())
        .and_then(|accessor| buffers::read_accessor::<[[f32; 4]; 4]>(buffers, accessor).ok())
    else {
        return Vec::new();
    };

    face_indices
        .into_iter()
        .zip(transforms)
        .map(|(face_index, t)| SerializablePiece {
            face_index,
            transform: cgmath::Matrix4::from(t),
        })
        .collect()
}
