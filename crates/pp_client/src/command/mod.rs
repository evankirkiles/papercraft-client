use std::{cell::RefCell, rc::Rc};

use pp_core::{Command, CommandError, CommandType, RedoError, State, UndoError};
use wasm_bindgen::JsValue;

use crate::command::sync::SyncConnectionConfig;

pub mod sync;

#[derive(Debug, Default)]
pub struct MultiplayerCommandStack {
    commands: pp_core::CommandStack,
    /// Optional configuration for a persistence module
    sync: Option<sync::SyncManager>,
}

impl MultiplayerCommandStack {
    /// Rolls back the latest command on the undo/redo stack
    pub fn undo(&mut self, state: &mut State) -> Result<(), UndoError> {
        if let Some(sync) = &self.sync {
            let command_i =
                self.commands.stack.len().wrapping_sub(self.commands.redos_available + 1);
            let Some(command) = self.commands.stack.get(command_i) else {
                return Err(UndoError::NoMoreUndos);
            };
            let _ = sync.send_command(command, true);
        };
        self.commands.undo(state)
    }

    /// Redoes the latest undone command on the undo/redo stack
    pub fn redo(&mut self, state: &mut State) -> Result<(), RedoError> {
        if let Some(sync) = &self.sync {
            let command_i = self.commands.stack.len().wrapping_sub(self.commands.redos_available);
            let Some(command) = self.commands.stack.get(command_i) else {
                return Err(RedoError::NoMoreRedos);
            };
            let _ = sync.send_command(command, false);
        };
        self.commands.redo(state)
    }

    /// Adds a new undoable command onto the undo / redo stack. This should be
    /// consistent with any corresponding modifications that happened on the mesh.
    pub fn add(&mut self, command: CommandType) {
        self.sync.as_mut().inspect(|sync| {
            sync.send_command(&command, false).expect("Failed to send command!");
        });
        self.commands.add(command);
    }

    /// Executes the command against the state and then adds the command onto
    /// the undo / redo stack. If you don't want to execute the command, just
    /// use `add`.
    pub fn execute(&mut self, state: &mut State, command: CommandType) -> Result<(), CommandError> {
        command.execute(state)?;
        self.add(command);
        Ok(())
    }

    // ---- MULTIPLAYER SYNC ----

    /// Connect to a multiplayer server for real-time synchronization
    pub fn subscribe(
        &mut self,
        state: Rc<RefCell<State>>,
        config: &SyncConnectionConfig,
    ) -> Result<(), JsValue> {
        let sync = sync::SyncManager::connect(state, config)?;
        self.sync = Some(sync);
        log::info!("Connected to multiplayer server");
        Ok(())
    }

    /// Disconnect from the multiplayer server
    pub fn unsubscribe(&mut self) -> Result<(), JsValue> {
        if let Some(sync) = self.sync.take() {
            sync.close()?;
            log::info!("Disconnected from multiplayer server");
        }
        Ok(())
    }

    /// Check if connected to a multiplayer server
    pub fn is_subscribed(&self) -> bool {
        self.sync.as_ref().map_or(false, |s| s.is_connected())
    }
}
