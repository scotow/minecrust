use crate::types;

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Chat {
    content: types::String
}

impl Chat {
    pub fn new(message: &str) -> Self {
        let content = serde_json::json!({
            "translate": "chat.type.announcement",
            "with": [
                {
                    "text": "Admin"
                },
                {
                    "text": message
                }
            ]
        }).to_string().into();
        Self {
            content
        }
    }
}