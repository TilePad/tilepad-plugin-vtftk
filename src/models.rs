use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Deserialize)]
pub struct ItemWithSounds {
    pub id: Uuid,
    pub config: serde_json::Value,
    pub impact_sounds_ids: Vec<Uuid>,
    pub windup_sounds_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct QueryById {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ThrowItemProperties {
    pub item: Option<Uuid>,
}
