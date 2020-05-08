use crate::types;

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Chat(types::String);

impl Chat {
    pub fn new(message: &str) -> Self {
        let content = serde_json::json!({
            "text": message
        }).to_string();
        Self(content.into())
    }

    pub fn user_message(sender: &str, message: &str) -> Self {
        let content = serde_json::json!({
            "translate": "[%s] %s",
            "with": [
                {
                    "text": sender
                },
                {
                    "text": message
                }
            ]
        }).to_string();
        Self(content.into())
    }

    fn connection_message(player: &str, state: &str) -> Self {
        let content = serde_json::json!({
            "translate": "%s %s the game.",
            "color": "yellow",
            "italic": "true",
            "with": [
                {
                    "text": player
                },
                {
                    "text": state
                }
            ]
        }).to_string();
        Self(content.into())
    }

    pub fn player_joined(player: &str) -> Self {
        Self::connection_message(player, "joined")
    }

    pub fn player_left(player: &str) -> Self {
        Self::connection_message(player, "left")
    }
}