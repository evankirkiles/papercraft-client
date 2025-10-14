use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    cut::{CutActionType, CutEdgeState},
    id::{self, Id},
    mesh::piece::Piece,
    MeshId,
};

use super::{Command, CommandError};

/// Cuts & joins edges, creating any resulting pieces from the operation. On each
/// cut, we save a before / after of any pieces on either side of each edge, as
/// well as a snapshot of any pieces involved in the operation (before OR after,
/// which is fine because no piece-internal data is changed as a result of cuts).
#[derive(Clone, Debug)]
pub struct CutEdgesCommand {
    pub action: CutActionType,
    pub edges: Vec<(MeshId, id::EdgeId)>,
    pub pieces: HashMap<(MeshId, id::PieceId), Piece>,
    pub before: HashMap<(MeshId, id::EdgeId), CutEdgeState>,
    pub after: HashMap<(MeshId, id::EdgeId), CutEdgeState>,
}

impl Serialize for CutEdgesCommand {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CutEdgesCommand", 5)?;
        state.serialize_field("action", &self.action)?;
        state.serialize_field("edges", &self.edges)?;

        // Serialize HashMaps as Vec of tuples
        let pieces_vec: Vec<_> = self.pieces.iter().collect();
        state.serialize_field("pieces", &pieces_vec)?;

        let before_vec: Vec<_> = self.before.iter().collect();
        state.serialize_field("before", &before_vec)?;

        let after_vec: Vec<_> = self.after.iter().collect();
        state.serialize_field("after", &after_vec)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for CutEdgesCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CutEdgesCommandHelper {
            action: CutActionType,
            edges: Vec<(MeshId, id::EdgeId)>,
            pieces: Vec<((MeshId, id::PieceId), Piece)>,
            before: Vec<((MeshId, id::EdgeId), CutEdgeState)>,
            after: Vec<((MeshId, id::EdgeId), CutEdgeState)>,
        }

        let helper = CutEdgesCommandHelper::deserialize(deserializer)?;
        Ok(CutEdgesCommand {
            action: helper.action,
            edges: helper.edges,
            pieces: helper.pieces.into_iter().collect(),
            before: helper.before.into_iter().collect(),
            after: helper.after.into_iter().collect(),
        })
    }
}

impl CutEdgesCommand {
    pub fn cut_edges(state: &mut crate::State, action: CutActionType) -> Self {
        // Figure out which edges of the selection are going to be cut.
        let edges: Vec<_> = state
            .selection
            .edges
            .iter()
            .filter(|(m_id, e_id)| {
                let mesh = &state.meshes[*m_id];
                if action == CutActionType::Cut {
                    mesh[*e_id].cut.is_none()
                        // Do not cut edges inside the selection border
                        && mesh.iter_edge_loops(*e_id).is_some_and(|mut walker| {
                            !walker.all(|l| {
                                state.selection.faces.contains(&(*m_id, mesh[l].f))
                            })
                        })
                } else {
                    mesh[*e_id].cut.is_some()
                }
            })
            .copied()
            .collect();
        // Build up the previous history around those edges. What were the
        // cut states, what were the existing pieces, etc.
        let mut pieces: HashMap<(MeshId, id::PieceId), Piece> = HashMap::new();
        let mut before: HashMap<(MeshId, id::EdgeId), CutEdgeState> = HashMap::new();
        let mut after: HashMap<(MeshId, id::EdgeId), CutEdgeState> = HashMap::new();
        // Iterate all these edges and put the "before" states into a map
        edges.iter().copied().for_each(|(m_id, e_id)| {
            let mesh = &state.meshes[m_id];
            let edge = mesh[e_id];
            let p_a = edge.l.and_then(|l| mesh[mesh[l].f].p);
            let p_b = edge.l.and_then(|l| mesh[mesh[mesh[l].radial_next].f].p);
            before.insert((m_id, e_id), CutEdgeState { cut: edge.cut, p_a, p_b });
            if let Some(p_a) = p_a {
                pieces.entry((m_id, p_a)).or_insert(mesh[p_a]);
            }
            if let Some(p_b) = p_b {
                pieces.entry((m_id, p_b)).or_insert(mesh[p_b]);
            }
        });
        // Perform the cut itself, without any backing history
        state.cut_edges(&edges[..], action, None);
        // Iterate all those edges again and put the "after" states into a map,
        // Plus any new pieces which may have been created by the cut
        edges.iter().copied().for_each(|(m_id, e_id)| {
            let mesh = &state.meshes[m_id];
            let edge = mesh[e_id];
            let p_a = edge.l.and_then(|l| mesh[mesh[l].f].p);
            let p_b = edge.l.and_then(|l| mesh[mesh[mesh[l].radial_next].f].p);
            after.insert((m_id, e_id), CutEdgeState { cut: edge.cut, p_a, p_b });
            if let Some(p_a) = p_a {
                pieces.entry((m_id, p_a)).or_insert(mesh[p_a]);
            }
            if let Some(p_b) = p_b {
                pieces.entry((m_id, p_b)).or_insert(mesh[p_b]);
            }
        });
        // Our command is completed - we have enough information to recreate the
        // states before and after an edge cut.
        Self { action, edges, pieces, before, after }
    }
}

impl Command for CutEdgesCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.pieces.iter().for_each(|(id, piece)| {
            // TODO: Increase capacity of pieces to accomodate new ones
            // log::info!("{:?}", state.meshes.get_mut(id.0).unwrap().pieces);
            state.meshes.get_mut(id.0).unwrap().pieces.insert(id.1.to_usize(), *piece);
        });
        state.cut_edges(&self.edges, self.action, Some(&self.after));
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.pieces.iter().for_each(|(id, piece)| {
            state.meshes.get_mut(id.0).unwrap().pieces.insert(id.1.to_usize(), *piece);
        });
        state.cut_edges(
            &self.edges,
            match self.action {
                CutActionType::Join => CutActionType::Cut,
                CutActionType::Cut => CutActionType::Join,
            },
            Some(&self.before),
        );
        Ok(())
    }
}
