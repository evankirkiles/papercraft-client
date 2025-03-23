use crate::{
    id::{self},
    mesh::MeshElementType,
    State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CutActionType {
    Uncut,
    Cut,
    Invert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CutMaskType {
    None,
    SelectionBorder,
}

impl State {
    /// Cuts multiple edges at once
    pub fn cut_edges(
        &mut self,
        edges: &[(id::MeshId, id::EdgeId)],
        action: CutActionType,
        mask: CutMaskType,
    ) {
        edges.iter().for_each(|id| {
            let (m_id, e_id) = *id;
            if action == CutActionType::Uncut
                || match mask {
                    CutMaskType::None => true,
                    CutMaskType::SelectionBorder => {
                        let mesh = &self.meshes[&m_id];
                        !mesh.iter_edge_loops(e_id).is_some_and(|mut walker| {
                            walker.all(|l| self.selection.faces.contains(&(m_id, mesh[l].f)))
                        })
                    }
                }
            {
                self.cut_edge(id, action)
            }
        });
    }

    /// Cuts a single edge.
    /// TODO: Make this update any necessary shapes
    pub fn cut_edge(&mut self, id: &(id::MeshId, id::EdgeId), action: CutActionType) {
        let (m_id, e_id) = id;
        let cut = match action {
            CutActionType::Uncut => false,
            CutActionType::Cut => true,
            CutActionType::Invert => !self.meshes[m_id][*e_id].is_cut,
        };
        if let Some(mesh) = self.meshes.get_mut(m_id) {
            mesh[*e_id].is_cut = cut;
            mesh.elem_dirty |= MeshElementType::EDGES;
        };
    }
}
