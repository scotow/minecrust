use serde::Serialize;

#[derive(Default, Debug, Clone)]
pub struct ServerDescription {
    pub version: Version,
    pub players: (u32, u32),
    pub description: String,
    pub icon: Option<Vec<u8>>,
}

impl ServerDescription {
    pub fn icon_data(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|i| format!("data:image/png;base64,{}", base64::encode(i)))
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
