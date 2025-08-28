use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cell::RefCell, rc::Rc};
use tilepad_plugin_sdk::{
    inspector::Inspector, plugin::Plugin, protocol::TileInteractionContext,
    session::PluginSessionHandle, tracing,
};
use uuid::Uuid;

/// Properties for the plugin itself
#[derive(Debug, Deserialize, Serialize)]
pub struct Properties {}

/// Messages from the inspector
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InspectorMessageIn {
    GetItems,
}

/// Messages to the inspector
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InspectorMessageOut {
    Items { items: Vec<ItemModel> },
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemModel {
    pub id: Uuid,
    pub name: String,
    pub config: ItemConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemConfig {
    pub image: ItemImageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemImageConfig {
    pub src: String,
    pub pixelate: bool,
}

#[derive(Deserialize)]
pub struct ItemWithSounds {
    pub id: Uuid,
    pub config: serde_json::Value,
    pub impact_sounds_ids: Vec<Uuid>,
    pub windup_sounds_ids: Vec<Uuid>,
}

impl Plugin for VtftkPlugin {
    fn on_properties(&self, _session: &PluginSessionHandle, _properties: serde_json::Value) {}

    fn on_inspector_open(&self, _session: &PluginSessionHandle, inspector: Inspector) {
        self.state.set_inspector(Some(inspector));
    }

    fn on_inspector_close(&self, _session: &PluginSessionHandle, _inspector: Inspector) {
        self.state.set_inspector(None);
    }

    fn on_inspector_message(
        &self,
        _session: &PluginSessionHandle,
        inspector: Inspector,
        message: serde_json::Value,
    ) {
        let message: InspectorMessageIn = match serde_json::from_value(message) {
            Ok(value) => value,
            Err(_) => return,
        };

        match message {
            InspectorMessageIn::GetItems => {
                tokio::spawn(async move {
                    let response = reqwest::get("http://localhost:58371/items").await.unwrap();
                    let items: Vec<ItemModel> = response.json().await.unwrap();

                    inspector
                        .send(InspectorMessageOut::Items { items })
                        .unwrap();
                });
            }
        }
    }

    fn on_tile_clicked(
        &self,
        _session: &PluginSessionHandle,
        ctx: TileInteractionContext,
        properties: serde_json::Value,
    ) {
        let action_id = ctx.action_id.as_str();
        let action = match Action::from_action(action_id, properties) {
            Some(Ok(value)) => value,
            Some(Err(cause)) => {
                tracing::error!(?cause, ?action_id, "failed to deserialize action");
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
                    None => return,
                };

                tokio::spawn(async move {
                    let client = reqwest::Client::new();

                    // Load the items
                    let items_response = client
                        .post("http://localhost:58371/items/query-by-id")
                        .json(&QueryById { ids: vec![item_id] })
                        .send()
                        .await
                        .unwrap();
                    let items_with_sounds: Vec<ItemWithSounds> =
                        items_response.json().await.unwrap();
                    let item_with_sound = items_with_sounds.first().unwrap();

                    // Collect all sound Ids
                    let sound_ids = item_with_sound
                        .impact_sounds_ids
                        .iter()
                        .chain(item_with_sound.windup_sounds_ids.iter())
                        .copied()
                        .collect::<Vec<Uuid>>();

                    // Load the sounds
                    let sounds_response = client
                        .post("http://localhost:58371/sounds/query-by-id")
                        .json(&QueryById { ids: sound_ids })
                        .send()
                        .await
                        .unwrap();
                    let sounds: Vec<serde_json::Value> = sounds_response.json().await.unwrap();

                    client
                        .post("http://localhost:58371/overlay/events")
                        .json(&json!({
                            "type": "ThrowItem",
                            "items": {
                                "items": [
                                    {
                                        "id": item_with_sound.id,
                                        "config": item_with_sound.config,
                                        "impact_sound_ids": item_with_sound.impact_sounds_ids,
                                        "windup_sound_ids": item_with_sound.windup_sounds_ids,
                                    }
                                ],
                                "sounds": sounds
                            },
                            "config": {
                                "type": "All",
                                "amount": 1,
                            }
                        }))
                        .send()
                        .await
                        .unwrap();
                });
            }
        }
    }
}

#[derive(Serialize)]
pub struct QueryById {
    ids: Vec<Uuid>,
}

pub enum Action {
    ThrowItem(ThrowItemProperties),
}

impl Action {
    pub fn from_action(
        action_id: &str,
        properties: serde_json::Value,
    ) -> Option<Result<Action, serde_json::Error>> {
        Some(match action_id {
            "throw_item" => serde_json::from_value(properties).map(Action::ThrowItem),
            _ => return None,
        })
    }
}

#[derive(Deserialize)]
pub struct ThrowItemProperties {
    pub item: Option<Uuid>,
}
