use serde::{Deserialize, Serialize};

use crate::select::{self, SelectionActionType};

use super::{Command, CommandError};

/// A modification of the current select state. Because there are many possible
/// side effects of these types of commands, we simply store before / after
/// snapshots of the select state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectCommand {
    pub after: Box<select::SelectionState>,
    pub before: Box<select::SelectionState>,
}

impl SelectCommand {
    /// Selects all the elements in the mesh
    pub fn select_all(state: &mut crate::State, action: SelectionActionType) -> Self {
        let before = Box::new(state.selection.clone());
        state.select_all(action);
        Self { before, after: Box::new(state.selection.clone()) }
    }
}

impl Command for SelectCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        state.selection = *self.after.clone();
        state.selection.is_dirty = true;
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        state.selection = *self.before.clone();
        state.selection.is_dirty = true;
        Ok(())
    }
}
