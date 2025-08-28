use crate::models::{ItemModel, ItemWithSounds, QueryById};
use serde_json::json;
use std::cell::{Cell, RefCell};
use tilepad_plugin_sdk::Inspector;
use uuid::Uuid;

pub struct State {
    port: Cell<u16>,
    client: reqwest::Client,
    inspector: RefCell<Option<Inspector>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            port: Cell::new(8533),
            client: Default::default(),
            inspector: RefCell::default(),
        }
    }
}

impl State {
    pub fn set_inspector(&self, inspector: Option<Inspector>) {
        *self.inspector.borrow_mut() = inspector;
    }

    pub fn set_port(&self, port: u16) {
        self.port.set(port);
    }

    pub async fn get_items(&self) -> anyhow::Result<Vec<ItemModel>> {
        let port = self.port.get();
        let response = reqwest::get(format!("http://127.0.0.1:{port}/items")).await?;
        let items: Vec<ItemModel> = response.json().await?;
        Ok(items)
    }

    pub async fn get_item_by_id(&self, item_id: Uuid) -> anyhow::Result<Option<ItemWithSounds>> {
        let port = self.port.get();
        let items_response = self
            .client
            .post(format!("http://127.0.0.1:{port}/items/query-by-id"))
            .json(&QueryById { ids: vec![item_id] })
            .send()
            .await
            .unwrap();
        let mut items_with_sounds: Vec<ItemWithSounds> = items_response.json().await?;

        Ok(items_with_sounds.pop())
    }

    pub async fn get_sounds_by_id(
        &self,
        sound_ids: Vec<Uuid>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let port = self.port.get();
        let sounds_response = self
            .client
            .post(format!("http://127.0.0.1:{port}/sounds/query-by-id"))
            .json(&QueryById { ids: sound_ids })
            .send()
            .await?;
        let sounds: Vec<serde_json::Value> = sounds_response.json().await?;

        Ok(sounds)
    }

    pub async fn throw_item(
        &self,
        item_with_sound: ItemWithSounds,
        sounds: Vec<serde_json::Value>,
    ) -> anyhow::Result<()> {
        let port = self.port.get();
        self.client
            .post(format!("http://127.0.0.1:{port}/overlay/events"))
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
            .await?;
        Ok(())
    }
}
