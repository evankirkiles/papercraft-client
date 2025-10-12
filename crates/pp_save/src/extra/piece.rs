use pp_core::{id::Id, State};
use serde::{Deserialize, Serialize};

/// Represents a piece in the unfolded mesh
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Piece {
    /// The mesh this piece belongs to
    pub mesh: u32,
    /// The root face index that defines this piece
    pub root_face: u32,
    /// The transformation matrix for this piece (column-major order)
    pub transform: [f32; 16],
    /// Unfold progress (0.0 = 3D, 1.0 = 2D)
    pub unfold_t: f32,
}

/// Converts pieces from pp_core State to PPR format
pub fn save_pieces(state: &State) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for (mesh_idx, (_mesh_id, mesh)) in state.meshes.iter().enumerate() {
        for piece_id in mesh.pieces.indices() {
            let piece_id = pp_core::id::PieceId::from_usize(piece_id);
            let piece = &mesh[piece_id];

            // Convert cgmath::Matrix4 to array (column-major order)
            let transform: [f32; 16] = [
                piece.transform.x.x,
                piece.transform.x.y,
                piece.transform.x.z,
                piece.transform.x.w,
                piece.transform.y.x,
                piece.transform.y.y,
                piece.transform.y.z,
                piece.transform.y.w,
                piece.transform.z.x,
                piece.transform.z.y,
                piece.transform.z.z,
                piece.transform.z.w,
                piece.transform.w.x,
                piece.transform.w.y,
                piece.transform.w.z,
                piece.transform.w.w,
            ];

            pieces.push(Piece {
                mesh: mesh_idx as u32,
                root_face: piece.f.to_usize() as u32,
                transform,
                unfold_t: piece.t,
            });
        }
    }

    pieces
}

pub fn load_pieces(state: &mut State, pieces: &Vec<Piece>) {
    todo!()
}
