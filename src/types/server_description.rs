use crate::game::World;
use piper::Arc;
use serde::Serialize;

#[derive(Clone)]
pub enum Value<T> {
    Direct(T),
    FromFn(Arc<Box<dyn Fn(&World) -> T>>),
}

impl<T> std::fmt::Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value")
    }
}

impl<T: Clone> Value<T> {
    pub fn get(&self, world: &World) -> T {
        match self {
            Self::Direct(v) => v.clone(),
            Self::FromFn(f) => f(world),
        }
    }
}

impl<T: Default> Default for Value<T> {
    fn default() -> Self {
        Self::Direct(Default::default())
    }
}

#[derive(Default, Debug, Clone)]
pub struct ServerDescription {
    pub version: Version,
    pub player_connected: Value<u32>,
    pub player_max: Value<u32>,
    pub description: String,
    pub icon: Option<Vec<u8>>,
}

impl ServerDescription {
    pub fn icon_data(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|i| format!("data:image/png;base64,{}", base64::encode(i)))
    }

    pub fn player_connected(&self, world: &World) -> u32 {
        self.player_connected.get(world)
    }

    pub fn player_max(&self, world: &World) -> u32 {
        self.player_max.get(world)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Version {
    name: &'static str,
    protocol: u16,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            name: "1.15.2",
            protocol: 578,
        }
    }
}
