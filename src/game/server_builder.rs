use super::map::generator::ChunkGenerator;
use super::world::World;
use crate::types::{ServerDescription, Version, Value};
use anyhow::Result;
use piper::Arc;

/// An helper function to easily create a server
#[derive(Clone)]
pub struct ServerBuilder {
    description: ServerDescription,
}

impl ServerBuilder {
    /// create a ServerBuilder with a default ServerDescription
    pub fn new() -> Self {
        Default::default()
    }

    /// create a ServerBuilder with your ServerDescription
    pub fn from_description(description: ServerDescription) -> Self {
        Self { description }
    }

    /// Set the version of your server in place
    pub fn set_version(&mut self, version: Version) {
        self.description.version = version;
    }

    /// Set the version of your server
    pub fn with_version(mut self, version: Version) -> Self {
        self.set_version(version);
        self
    }

    /// Set the max players of your server in place
    pub fn set_player_max(&mut self, player_max: u32) {
        self.description.player_max = Value::Direct(player_max);
    }

    /// Set the max players of your server
    pub fn with_player_max(mut self, player_max: u32) -> Self {
        self.set_player_max(player_max);
        self
    }
 
    /// Set the max players of your server in place from a function or closure
    pub fn set_player_max_from_fn(&mut self, player_max: impl Fn(&World) -> u32 + 'static) {
        self.description.player_max = Value::FromFn(Arc::new(Box::new(player_max)));
    }

    /// Set the max players of your server from a function or closure
    pub fn with_player_max_from_fn(mut self, player_max: impl Fn(&World) -> u32 + 'static) -> Self{
        self.set_player_max_from_fn(player_max);
        self
    }

    /// Set the number of connected players of your server in place
    pub fn set_player_connected(&mut self, player_connected: u32) {
        self.description.player_connected = Value::Direct(player_connected);
    }

    /// Set the number of connected players of your server
    pub fn with_player_connected(mut self, player_connected: u32) -> Self {
        self.set_player_connected(player_connected);
        self
    }
 
    /// Set the number of connected players of your server in place from a function or closure
    pub fn set_player_connected_from_fn(&mut self, player_connected: impl Fn(&World) -> u32 + 'static) {
        self.description.player_connected = Value::FromFn(Arc::new(Box::new(player_connected)));
    }

    /// Set the number of connected players of your server from a function or closure
    pub fn with_player_connected_from_fn(mut self, player_connected: impl Fn(&World) -> u32 + 'static) -> Self{
        self.set_player_connected_from_fn(player_connected);
        self
    }

    /// Set the description of your server in place
    pub fn set_description(&mut self, description: String) {
        self.description.description = description;
    }

    /// Set the description of your server
    pub fn with_description(mut self, description: String) -> Self {
        self.set_description(description);
        self
    }

    /// Set the icon of your server in place
    pub fn set_icon(&mut self, icon: Vec<u8>) {
        self.description.icon = Some(icon);
    }

    /// Set the icon of your server
    pub fn with_icon(mut self, icon: Vec<u8>) -> Self {
        self.set_icon(icon);
        self
    }

    /// Set the icon of your server from its path in place
    pub fn set_icon_from_path(&mut self, icon: &str) -> Result<()> {
        let icon = std::fs::read(icon)?;
        self.description.icon = Some(icon);
        Ok(())
    }

    /// Set the icon of your server
    pub fn with_icon_from_path(mut self, icon: &str) -> Result<Self> {
        self.set_icon_from_path(icon)?;
        Ok(self)
    }

    /// Build a World from the provided generator
    pub async fn build<G>(self, generator: G) -> World
    where
        G: ChunkGenerator + Sync + Send + 'static,
    {
        World::new(self.description, generator).await
    }

    /// Build a World from the provided generator, put it in the heap,
    /// and provide an &'static reference to it
    pub async fn build_leak<G>(self, generator: G) -> &'static World
    where
        G: ChunkGenerator + Sync + Send + 'static,
    {
        Box::leak(Box::new(self.build(generator).await))
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self {
            description: ServerDescription::default(),
        }
    }
}
