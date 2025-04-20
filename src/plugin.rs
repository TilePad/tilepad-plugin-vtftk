use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use tilepad_plugin_sdk::{
    inspector::Inspector, plugin::Plugin, protocol::TileInteractionContext,
    session::PluginSessionHandle,
};
use tokio::task::spawn_local;

/// Properties for the plugin itself
#[derive(Debug, Deserialize, Serialize)]
pub struct Properties {}

/// Messages from the inspector
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InspectorMessageIn {}

/// Messages to the inspector
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InspectorMessageOut {}

/// Option for a select dropdown menu
#[derive(Deserialize, Serialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
}

#[derive(Default)]
pub struct VtftkPlugin {
    state: Rc<State>,
}

#[derive(Default)]
pub struct State {
    inspector: RefCell<Option<Inspector>>,
}

impl State {
    fn set_inspector(&self, inspector: Option<Inspector>) {
        *self.inspector.borrow_mut() = inspector;
    }
}

impl VtftkPlugin {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Plugin for VtftkPlugin {
    fn on_properties(&self, session: &PluginSessionHandle, properties: serde_json::Value) {
        let session = session.clone();
    }

    fn on_inspector_open(&self, _session: &PluginSessionHandle, inspector: Inspector) {
        self.state.set_inspector(Some(inspector));
    }

    fn on_inspector_close(&self, _session: &PluginSessionHandle, _inspector: Inspector) {
        self.state.set_inspector(None);
    }

    fn on_inspector_message(
        &self,
        session: &PluginSessionHandle,
        inspector: Inspector,
        message: serde_json::Value,
    ) {
        let message: InspectorMessageIn = match serde_json::from_value(message) {
            Ok(value) => value,
            Err(_) => return,
        };

        let session = session.clone();

        match message {}
    }

    fn on_tile_clicked(
        &self,
        _session: &PluginSessionHandle,
        ctx: TileInteractionContext,
        properties: serde_json::Value,
    ) {
    }
}
