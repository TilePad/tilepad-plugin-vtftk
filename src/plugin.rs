use crate::{
    action::Action,
    messages::{InspectorMessageIn, InspectorMessageOut},
    state::State,
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tilepad_plugin_sdk::{Inspector, Plugin, PluginSessionHandle, TileInteractionContext, tracing};
use tokio::task::spawn_local;
use uuid::Uuid;

/// Properties for the plugin itself
#[derive(Debug, Deserialize, Serialize)]
pub struct Properties {
    port: Option<u16>,
}

#[derive(Default)]
pub struct VtftkPlugin {
    state: Rc<State>,
}

impl Plugin for VtftkPlugin {
    fn on_properties(&mut self, _session: &PluginSessionHandle, properties: serde_json::Value) {
        let properties: Properties = match serde_json::from_value(properties) {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(?error, "failed to parse properties");
                return;
            }
        };

        self.state.set_port(properties.port.unwrap_or(8533));
    }

    fn on_inspector_open(&mut self, _session: &PluginSessionHandle, inspector: Inspector) {
        self.state.set_inspector(Some(inspector));
    }

    fn on_inspector_close(&mut self, _session: &PluginSessionHandle, _inspector: Inspector) {
        self.state.set_inspector(None);
    }

    fn on_inspector_message(
        &mut self,
        session: &PluginSessionHandle,
        inspector: Inspector,
        message: serde_json::Value,
    ) {
        let message: InspectorMessageIn = match serde_json::from_value(message) {
            Ok(value) => value,
            Err(_) => return,
        };

        let state = self.state.clone();

        match message {
            InspectorMessageIn::GetItems => {
                spawn_local(async move {
                    let items = match state.get_items().await {
                        Ok(value) => value,
                        Err(error) => {
                            tracing::error!(?error, "failed to get items");
                            return;
                        }
                    };

                    _ = inspector.send(InspectorMessageOut::Items { items });
                });
            }
            InspectorMessageIn::PortChanged { port } => {
                state.set_port(port);
                _ = session.set_properties(Properties { port: Some(port) });
            }
        }
    }

    fn on_tile_clicked(
        &mut self,
        _session: &PluginSessionHandle,
        ctx: TileInteractionContext,
        properties: serde_json::Value,
    ) {
        let action_id = ctx.action_id.as_str();
        let action = match Action::from_action(action_id, properties) {
            Some(Ok(value)) => value,
            Some(Err(error)) => {
                tracing::error!(?error, ?action_id, "failed to deserialize action");
                return;
            }
            None => {
                tracing::debug!(?action_id, "unknown tile action requested");
                return;
            }
        };

        match action {
            Action::ThrowItem(properties) => {
                let item_id = match properties.item {
                    Some(value) => value,

                    // Nothing to throw
                    None => return,
                };

                let state = self.state.clone();

                spawn_local(async move {
                    let item_with_sound = match state.get_item_by_id(item_id).await {
                        Ok(Some(value)) => value,
                        Ok(None) => return,
                        Err(error) => {
                            tracing::error!(?error, "failed to get item by id");
                            return;
                        }
                    };

                    tracing::debug!(?item_with_sound, "got item to throw");

                    // Collect all sound IDs
                    let sound_ids = item_with_sound
                        .impact_sounds_ids
                        .iter()
                        .chain(item_with_sound.windup_sounds_ids.iter())
                        .copied()
                        .collect::<Vec<Uuid>>();

                    tracing::debug!(?sound_ids, "sound ids to load");

                    let sounds = match state.get_sounds_by_id(sound_ids).await {
                        Ok(value) => value,
                        Err(error) => {
                            tracing::error!(?error, "failed to get sounds by id");
                            return;
                        }
                    };

                    tracing::debug!(?sounds, "sounds");

                    if let Err(error) = state.throw_item(item_with_sound, sounds).await {
                        tracing::error!(?error, "failed to throw item");
                    }
                });
            }
        }
    }
}
