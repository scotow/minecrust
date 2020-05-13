use crate::types::chat::Chat;
use serde_json::Value;
use std::collections::HashSet;

/// ```
/// use minecrust::types::chat::*;
///
/// let message =
///     ChatBuilder::template("%s, %s!".into())
///         .add_attribute(Attribute::Bold)
///         .add_component(
///             ChatComponent::new("Hello".into())
///                 .add_attribute(Attribute::Underlined)
///                 .add_attribute(Attribute::Color(Color::Pink))
///         )
///         .add_component(
///             ChatComponent::new("World".into())
///                 .add_attribute(Attribute::Italic)
///                 .add_attribute(Attribute::Color(Color::Red))
///         )
///         .build();
/// ```

pub struct ChatBuilder {
    content_type: ContentType,
    attributes: HashSet<Attribute>,
    components: Vec<ChatComponent>,
}

impl ChatBuilder {
    fn new(content: ContentType) -> Self {
        Self {
            content_type: content,
            attributes: HashSet::new(),
            components: Vec::new(),
        }
    }

    pub fn text(s: String) -> Self {
        Self::new(ContentType::Text(s))
    }

    pub fn template(s: String) -> Self {
        Self::new(ContentType::Template(s))
    }

    pub fn add_attribute(mut self, a: Attribute) -> Self {
        self.attributes.insert(a);
        self
    }

    pub fn add_component(mut self, c: ChatComponent) -> Self {
        self.components.push(c);
        self
    }

    pub fn build(self) -> Chat {
        let mut map = serde_json::Map::new();

        // Content type.
        use ContentType::*;
        match &self.content_type {
            Text(s) => map.insert("text".to_string(), Value::String(s.clone())),
            Template(t) => map.insert("translate".to_string(), Value::String(t.clone())),
        };

        // Attributes.
        for attr in self.attributes {
            map.insert(attr.key(), attr.value());
        }

        if !self.components.is_empty() {
            let key = match self.content_type {
                Text(_) => "extra",
                Template(_) => "with",
            }
            .to_string();
            let components = Value::Array(
                self.components
                    .into_iter()
                    .map(ChatComponent::build)
                    .collect(),
            );

            map.insert(key, components);
        }

        Chat::raw(Value::Object(map))
    }
}

enum ContentType {
    Text(String),
    Template(String),
}

pub struct ChatComponent {
    text: String,
    attributes: HashSet<Attribute>,
}

impl ChatComponent {
    pub fn new(text: String) -> Self {
        Self {
            text,
            attributes: HashSet::new(),
        }
    }

    pub fn add_attribute(mut self, a: Attribute) -> Self {
        self.attributes.insert(a);
        self
    }

    pub(self) fn build(self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("text".to_string(), Value::String(self.text));
        for attr in self.attributes {
            map.insert(attr.key(), attr.value());
        }
        Value::Object(map)
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Attribute {
    Bold,
    Italic,
    Underlined,
    Strikethrough,
    Obfuscated,
    Color(Color),
}

impl Attribute {
    pub(self) fn key(&self) -> String {
        use Attribute::*;
        match self {
            Bold => "bold",
            Italic => "italic",
            Underlined => "underlined",
            Strikethrough => "strikethrough",
            Obfuscated => "obfuscated",
            Color(_) => "color",
        }
        .to_string()
    }

    pub(self) fn value(&self) -> Value {
        use Attribute::*;
        match self {
            Bold => "true",
            Italic => "true",
            Underlined => "true",
            Strikethrough => "true",
            Obfuscated => "true",
            Color(c) => c.value(),
        }
        .to_string()
        .into()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    Purple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Cyan,
    Red,
    Pink,
    Yellow,
    White,
    Reset,
}

impl Color {
    pub(self) fn value(&self) -> &'static str {
        use Color::*;
        match self {
            Black => "black",
            DarkBlue => "dark_blue",
            DarkGreen => "dark_green",
            DarkCyan => "dark_aqua",
            DarkRed => "dark_red",
            Purple => "dark_purple",
            Gold => "gold",
            Gray => "gray",
            DarkGray => "dark_gray",
            Blue => "blue",
            Green => "green",
            Cyan => "aqua",
            Red => "red",
            Pink => "light_purple",
            Yellow => "yellow",
            White => "white",
            Reset => "reset",
        }
    }
}
