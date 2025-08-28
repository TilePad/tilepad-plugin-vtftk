use crate::models::ThrowItemProperties;

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
