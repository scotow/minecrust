use crate::types;
use crate::types::chat::{Attribute, ChatBuilder, ChatComponent, Color};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Chat(types::String);

impl Chat {
    pub(super) fn raw(json: serde_json::Value) -> Self {
        Self(json.to_string().into())
    }

    pub fn new(message: &str) -> Self {
        ChatBuilder::text(message.into()).build()
    }

    pub fn user_message(sender: &str, message: &str) -> Self {
        ChatBuilder::template("[%s] %s".into())
            .add_component(ChatComponent::new(sender.into()))
            .add_component(ChatComponent::new(message.into()))
            .build()
    }

    fn connection_message(player: &str, state: &str) -> Self {
        ChatBuilder::template("%s %s the game.".into())
            .add_attribute(Attribute::Italic)
            .add_attribute(Attribute::Color(Color::Yellow))
            .add_component(ChatComponent::new(player.into()))
            .add_component(ChatComponent::new(state.into()))
            .build()
    }

    pub fn player_joined(player: &str) -> Self {
        Self::connection_message(player, "joined")
    }

    pub fn player_left(player: &str) -> Self {
        Self::connection_message(player, "left")
    }
}
