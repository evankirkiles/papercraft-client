use make_cuts::MakeCutsCommand;
use scale_mesh::ScaleMeshCommand;
use select_elements::SelectCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use transform_mesh::TransformMeshCommand;
use transform_pieces::TransformPiecesCommand;
use update_flaps::UpdateFlapsCommand;

use crate::{clear_cuts::ClearCutsCommand, id, id::EdgeId, mesh::cut::FlapPosition, MeshId, State};

pub mod clear_cuts;
pub mod make_cuts;
pub mod scale_mesh;
pub mod select_elements;
pub mod transform_mesh;
pub mod transform_pieces;
pub mod update_flaps;

/// Buckets `(mesh, element)` pairs by mesh, preserving the order they came in.
pub(crate) fn group_by_mesh<T: Copy>(ids: &[(MeshId, T)]) -> Vec<(MeshId, Vec<T>)> {
    let mut grouped: Vec<(MeshId, Vec<T>)> = Vec::new();
    for (m_id, id) in ids {
        match grouped.iter_mut().find(|(existing, _)| existing == m_id) {
            Some((_, bucket)) => bucket.push(*id),
            None => grouped.push((*m_id, vec![*id])),
        }
    }
    grouped
}

/// Every live cut's flap position across the whole document. Cutting rewrites
/// flaps as a side effect (see `Mesh::assign_piece_flaps`), so commands which
/// cut snapshot this before and after and store the difference, letting undo put
/// back exactly what was there instead of re-deriving it and losing manual swaps.
pub(crate) fn snapshot_flaps(state: &State) -> HashMap<(MeshId, EdgeId), FlapPosition> {
    state
        .meshes
        .iter()
        .flat_map(|(m_id, mesh)| {
            mesh.cuts.iter().map(move |(e_id, cut)| ((m_id, *e_id), cut.flap_position))
        })
        .collect()
}

/// The flaps which changed between `before` and the current state, as
/// `(before, after)` lists ready to be replayed by a command's execute /
/// rollback. Cuts which didn't exist in `before` are recorded at their default,
/// which is where a rollback's `clear_cut` leaves them anyway.
pub(crate) fn diff_flaps(
    before: &HashMap<(MeshId, EdgeId), FlapPosition>,
    state: &State,
) -> (Vec<((MeshId, EdgeId), FlapPosition)>, Vec<((MeshId, EdgeId), FlapPosition)>) {
    let mut after: Vec<_> = snapshot_flaps(state)
        .into_iter()
        .filter(|(id, flap_position)| {
            u8::from(before.get(id).copied().unwrap_or_default()) != u8::from(*flap_position)
        })
        .collect();
    after.sort_by_key(|(id, _)| *id);
    let before =
        after.iter().map(|(id, _)| (*id, before.get(id).copied().unwrap_or_default())).collect();
    (before, after)
}

/// Replays a recorded flap layout onto the state. Run after a command's cuts
/// have been applied, so the recorded positions win over whatever the automatic
/// assignment inside `make_cut` / `clear_cut` came up with.
pub(crate) fn apply_flaps(state: &mut State, flaps: &[((MeshId, EdgeId), FlapPosition)]) {
    flaps.iter().for_each(|((m_id, e_id), flap_position)| {
        if let Some(mesh) = state.meshes.get_mut(*m_id) {
            mesh.set_cut_flap(*e_id, *flap_position);
        }
    });
}

/// The roots of the pieces sitting on either side of each of `edges`. A piece
/// *is* its root face — the transform and the unfold origin both hang off it —
/// so commands which cut record this before and after and replay it, letting a
/// piece which gets merged away and split back out land where it was instead of
/// at whichever face the radial iteration happens to reach first.
pub(crate) fn snapshot_roots(
    state: &State,
    edges: &[(MeshId, EdgeId)],
) -> Vec<(MeshId, id::FaceId)> {
    let mut roots: Vec<_> = edges
        .iter()
        .filter_map(|(m_id, e_id)| {
            let mesh = state.meshes.get(*m_id)?;
            let (f_1, f_2) = mesh.get_adjacent_two_faces(*e_id)?;
            Some([(*m_id, mesh[f_1].p), (*m_id, mesh[f_2].p)])
        })
        .flatten()
        .filter_map(|(m_id, p)| p.map(|p| (m_id, p)))
        .collect();
    // Sorted and deduped so peers and the server replay the same roots
    roots.sort();
    roots.dedup();
    roots
}

/// Replays a recorded set of piece roots onto the state. Run after a command's
/// cuts have been applied, so the pieces around them are rooted where they were
/// rather than wherever `make_cut` / `clear_cut` re-derived. Re-rooting an
/// already correctly rooted piece is a no-op, and a region which was a piece
/// when this was recorded is acyclic again now that the cuts match, so a failed
/// expansion means nothing was there to restore.
pub(crate) fn apply_roots(state: &mut State, roots: &[(MeshId, id::FaceId)]) {
    roots.iter().for_each(|(m_id, f_id)| {
        if let Some(mesh) = state.meshes.get_mut(*m_id) {
            let _ = mesh.expand_piece(*f_id);
        }
    });
}

pub enum UndoError {
    NoMoreUndos,
    Failure(CommandError),
}

pub enum RedoError {
    NoMoreRedos,
    Failure(CommandError),
}

#[derive(Debug, Clone, Default)]
pub struct CommandStack {
    /// The undo/redo stack
    pub stack: Vec<CommandType>,
    /// How many times you can redo
    pub redos_available: usize,
}

impl CommandStack {
    /// Rolls back the latest command on the undo/redo stack
    pub fn undo(&mut self, state: &mut State) -> Result<(), UndoError> {
        let command_i = self.stack.len().wrapping_sub(self.redos_available + 1);
        let Some(command) = self.stack.get(command_i) else {
            return Err(UndoError::NoMoreUndos);
        };
        command.rollback(state).map_err(UndoError::Failure)?;
        self.redos_available += 1;
        Ok(())
    }

    /// Redoes the latest undone command on the undo/redo stack
    pub fn redo(&mut self, state: &mut State) -> Result<(), RedoError> {
        let command_i = self.stack.len().wrapping_sub(self.redos_available);
        let Some(command) = self.stack.get(command_i) else {
            return Err(RedoError::NoMoreRedos);
        };
        command.execute(state).map_err(RedoError::Failure)?;
        self.redos_available -= 1;
        Ok(())
    }

    /// Adds a new undoable command onto the undo / redo stack. This should be
    /// consistent with any corresponding modifications that happened on the mesh.
    pub fn add(&mut self, command: CommandType) {
        // Clear any redoable commands from the stack
        if self.redos_available != 0 {
            let end = self.stack.len();
            let start = end - self.redos_available;
            self.stack.drain(start..end);
            self.redos_available = 0;
        }
        self.stack.push(command);
    }

    /// Executes the command against the state and then adds the command onto
    /// the undo / redo stack. If you don't want to execute the command, just
    /// use `add`.
    pub fn execute(&mut self, state: &mut State, command: CommandType) -> Result<(), CommandError> {
        command.execute(state)?;
        self.add(command);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandError {
    Unknown,
}

/// A `Command` bridges the gap between user IO and stateful operations. Once
/// a user has confirmed some sort of action on the model, that action is encoded
/// as a command and stored locally, enabling undo/redo, syncing with the server
/// for autosave, and
///
/// Because we use raw state on the server without keeping track of selections,
/// command `execution` and `rollback` should never have any dependencies on select
/// states so commands can be run anywhere.
pub trait Command {
    fn execute(&self, state: &mut State) -> Result<(), CommandError>;
    fn rollback(&self, state: &mut State) -> Result<(), CommandError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    Select(SelectCommand),
    TransformPieces(TransformPiecesCommand),
    TransformMesh(TransformMeshCommand),
    ScaleMesh(ScaleMeshCommand),
    ClearCuts(ClearCutsCommand),
    MakeCuts(MakeCutsCommand),
    UpdateFlaps(UpdateFlapsCommand),
}

impl Command for CommandType {
    fn execute(&self, state: &mut State) -> Result<(), CommandError> {
        match self {
            CommandType::Select(cmd) => cmd.execute(state),
            CommandType::TransformPieces(cmd) => cmd.execute(state),
            CommandType::TransformMesh(cmd) => cmd.execute(state),
            CommandType::ScaleMesh(cmd) => cmd.execute(state),
            CommandType::ClearCuts(cmd) => cmd.execute(state),
            CommandType::MakeCuts(cmd) => cmd.execute(state),
            CommandType::UpdateFlaps(cmd) => cmd.execute(state),
        }
    }

    fn rollback(&self, state: &mut State) -> Result<(), CommandError> {
        match self {
            CommandType::Select(cmd) => cmd.rollback(state),
            CommandType::TransformPieces(cmd) => cmd.rollback(state),
            CommandType::TransformMesh(cmd) => cmd.rollback(state),
            CommandType::ScaleMesh(cmd) => cmd.rollback(state),
            CommandType::ClearCuts(cmd) => cmd.rollback(state),
            CommandType::MakeCuts(cmd) => cmd.rollback(state),
            CommandType::UpdateFlaps(cmd) => cmd.rollback(state),
        }
    }
}
