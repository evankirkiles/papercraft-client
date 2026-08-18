use serde::{Deserialize, Serialize};

use crate::{
    id::{self},
    mesh::cut::{CutUpdate, FlapPosition},
    MeshId,
};

use super::{
    apply_flaps, apply_roots, diff_flaps, group_by_mesh, snapshot_flaps, snapshot_roots, Command,
    CommandError,
};

/// Cuts & joins edges, creating any resulting pieces from the operation. On each
/// cut, we save a before / after of any pieces on either side of each edge, as
/// well as a snapshot of any pieces involved in the operation (before OR after,
/// which is fine because no piece-internal data is changed as a result of cuts).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MakeCutsCommand {
    pub edges: Vec<(MeshId, id::EdgeId)>,
    /// The flaps which changing these cuts moved, recorded so undo restores the
    /// exact previous layout rather than re-deriving one and dropping any manual
    /// swaps the user made. See `Mesh::assign_piece_flaps`.
    #[serde(default)]
    pub flaps_before: Vec<((MeshId, id::EdgeId), FlapPosition)>,
    #[serde(default)]
    pub flaps_after: Vec<((MeshId, id::EdgeId), FlapPosition)>,
    /// The roots of the pieces around these edges, recorded so undo / redo puts
    /// a piece back on the face it was rooted at. A piece's transform and its
    /// unfold origin both hang off its root face, so re-deriving the root
    /// instead would drop wherever the user had moved the piece to.
    #[serde(default)]
    pub roots_before: Vec<(MeshId, id::FaceId)>,
    #[serde(default)]
    pub roots_after: Vec<(MeshId, id::FaceId)>,
}

impl MakeCutsCommand {
    pub fn from_select(state: &mut crate::State) -> Self {
        let flaps = snapshot_flaps(state);
        let mut cmd = Self {
            flaps_before: Vec::new(),
            flaps_after: Vec::new(),
            roots_before: Vec::new(),
            roots_after: Vec::new(),
            edges: state
                .selection
                .edges
                .iter()
                .filter(|(m_id, e_id)| {
                    let mesh = &state.meshes[*m_id];
                    mesh.cuts.get(e_id).is_none_or(|cut| cut.is_dead)
                        && mesh.iter_edge_loops(*e_id).is_some_and(|mut walker| {
                            !walker.all(|l| state.selection.faces.contains(&(*m_id, mesh[l].f)))
                        })
                })
                .copied()
                .collect(),
        };
        cmd.roots_before = snapshot_roots(state, &cmd.edges);
        cmd.cut_forward(state, CutUpdate::PiecesAndFlaps);
        cmd.roots_after = snapshot_roots(state, &cmd.edges);
        (cmd.flaps_before, cmd.flaps_after) = diff_flaps(&flaps, state);
        cmd
    }

    /// Applies this command's cuts in forward order. `update` lets `execute`
    /// skip the automatic flap assignment: it replays `flaps_after` instead, so
    /// guessing first would only be work to throw away.
    fn cut_forward(&self, state: &mut crate::State, update: CutUpdate) {
        for (mesh_id, edge_ids) in group_by_mesh(&self.edges) {
            if let Some(mesh) = state.meshes.get_mut(mesh_id) {
                edge_ids.iter().for_each(|e_id| mesh.make_cut(*e_id, update));
            }
        }
    }

    /// Undoes this command's cuts, in reverse order.
    fn cut_backward(&self, state: &mut crate::State, update: CutUpdate) {
        for (mesh_id, edge_ids) in group_by_mesh(&self.edges) {
            if let Some(mesh) = state.meshes.get_mut(mesh_id) {
                edge_ids.iter().rev().for_each(|e_id| mesh.clear_cut(e_id, update));
            }
        }
    }
}

impl Command for MakeCutsCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.cut_forward(state, CutUpdate::PiecesOnly);
        apply_roots(state, &self.roots_after);
        apply_flaps(state, &self.flaps_after);
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.cut_backward(state, CutUpdate::PiecesOnly);
        apply_roots(state, &self.roots_before);
        apply_flaps(state, &self.flaps_before);
        Ok(())
    }
}
