use crate::select::{self};

use super::{Command, CommandError};

/// A modification of the current select state. Because there are many possible
/// side effects of these types of commands, we simply store before / after
/// snapshots of the select state.
#[derive(Clone, Debug)]
pub struct SelectCommand {
    pub after: select::SelectionState,
    pub before: select::SelectionState,
}

impl Command for SelectCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        state.selection = self.after.clone();
        state.selection.is_dirty = true;
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        state.selection = self.before.clone();
        state.selection.is_dirty = true;
        Ok(())
    }
}
