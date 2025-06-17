use select::SelectCommand;
use transform_piece::TransformPiecesCommand;

use crate::State;

pub mod select;
pub mod transform_piece;

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

pub enum CommandError {
    Unknown,
}

/// The base trait implemented by all commands
pub trait Command {
    fn execute(&self, state: &mut State) -> Result<(), CommandError>;
    fn rollback(&self, state: &mut State) -> Result<(), CommandError>;
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Select(SelectCommand),
    TransformPieces(TransformPiecesCommand),
}

impl Command for CommandType {
    fn execute(&self, state: &mut State) -> Result<(), CommandError> {
        match self {
            CommandType::Select(cmd) => cmd.execute(state),
            CommandType::TransformPieces(cmd) => cmd.execute(state),
        }
    }

    fn rollback(&self, state: &mut State) -> Result<(), CommandError> {
        match self {
            CommandType::Select(cmd) => cmd.rollback(state),
            CommandType::TransformPieces(cmd) => cmd.rollback(state),
        }
    }
}
