use pp_core::{Command, CommandType, State};
use pp_protocol::{ClientMessage, ServerMessage};
use pp_save::{load::Loadable, SaveFile};
use std::{cell::RefCell, io::Cursor, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

#[wasm_bindgen]
pub struct SyncConnectionConfig {
    /// The hostname of the server
    server_url: String,
    /// The document ID on the server
    doc_id: String,
}

#[wasm_bindgen]
impl SyncConnectionConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(server_url: String, doc_id: String) -> Self {
        Self { server_url, doc_id }
    }
}

/// Manages WebSocket connection to the pp_server for real-time sync
#[derive(Debug)]
pub struct SyncManager {
    ws: WebSocket,
}

impl SyncManager {
    /// Connect to a pp_server WebSocket endpoint for a specific document
    pub fn connect(
        state: Rc<RefCell<State>>,
        config: &SyncConnectionConfig,
    ) -> Result<Self, JsValue> {
        let ws_url = format!("{}/documents/{}", config.server_url, config.doc_id);
        log::info!("Connecting to WebSocket: {}", ws_url);

        let ws = WebSocket::new(&ws_url)?;

        // Clone references for closures
        let state_clone = Rc::clone(&state);
        let doc_id_clone = config.doc_id.clone();

        // Handle incoming messages from server
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = text.into();
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(ServerMessage::Joined {
                        state: state_bytes, version, client_count, ..
                    }) => {
                        log::info!(
                            "Joined document (version: {}, clients: {})",
                            version,
                            client_count
                        );
                        // Load initial state from server
                        if let Ok(save_file) = SaveFile::from_reader(Cursor::new(state_bytes)) {
                            if let Ok(loaded_state) = State::load(save_file) {
                                state_clone.replace(loaded_state);
                                log::info!("Loaded initial state from server");
                            }
                        }
                    }
                    Ok(ServerMessage::Command { client_id, command, rollback, .. }) => {
                        log::info!("Received command from {}: {:?}", client_id, command);
                        // Apply command from another client
                        let mut state = state_clone.borrow_mut();
                        if let Err(e) = match rollback {
                            true => command.rollback(&mut state),
                            false => command.execute(&mut state),
                        } {
                            log::error!("Failed to apply remote command: {:?}", e);
                        }
                    }
                    Ok(ServerMessage::StateSync { state, version }) => {
                        log::info!("Received state sync (version: {})", version);
                        if let Ok(save_file) = SaveFile::from_reader(Cursor::new(state)) {
                            if let Ok(loaded_state) = State::load(save_file) {
                                state_clone.replace(loaded_state);
                            }
                        }
                    }
                    Ok(ServerMessage::ClientJoined { client_id, client_count }) => {
                        log::info!("Client {} joined ({} total)", client_id, client_count);
                    }
                    Ok(ServerMessage::ClientLeft { client_id, client_count }) => {
                        log::info!("Client {} left ({} remaining)", client_id, client_count);
                    }
                    Err(e) => {
                        log::error!("Failed to parse server message: {:?}", e);
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        let on_error = Closure::wrap(Box::new(move |e: ErrorEvent| {
            log::error!("WebSocket error: {:?}", e);
        }) as Box<dyn FnMut(ErrorEvent)>);

        let on_close = Closure::wrap(Box::new(move |e: CloseEvent| {
            log::info!("WebSocket closed: code={}, reason={}", e.code(), e.reason());
        }) as Box<dyn FnMut(CloseEvent)>);

        // Set event handlers
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        // Keep all the closures alive
        on_message.forget();
        on_error.forget();
        on_close.forget();

        // Set up onopen to send Join message
        let ws_clone = ws.clone();
        let on_open = Closure::once(Box::new(move || {
            log::info!("WebSocket connected, sending Join message");
            let join_msg = ClientMessage::Join { doc_id: doc_id_clone };
            if let Ok(json) = serde_json::to_string(&join_msg) {
                let _ = ws_clone.send_with_str(&json);
            }
        }) as Box<dyn FnOnce()>);

        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget(); // Keep the closure alive

        Ok(Self { ws })
    }

    /// Send a command to the server
    pub fn send_command(&self, command: &CommandType, rollback: bool) -> Result<(), JsValue> {
        // log::info!("{:?}", command);
        let msg = ClientMessage::Command { command: command.clone(), rollback };
        let json = serde_json::to_string(&msg).map_err(|e| {
            log::error!("{:?}", e);
            JsValue::from_str(&format!("Failed to serialize command: {:?}", e))
        })?;
        self.ws.send_with_str(&json)?;
        Ok(())
    }

    /// Check if the WebSocket is currently connected
    pub fn is_connected(&self) -> bool {
        self.ws.ready_state() == WebSocket::OPEN
    }

    /// Close the WebSocket connection
    pub fn close(&self) -> Result<(), JsValue> {
        self.ws.close()
    }
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        let _ = self.ws.close();
    }
}
