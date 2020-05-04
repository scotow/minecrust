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
            "translate": "chat.type.text",
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
}