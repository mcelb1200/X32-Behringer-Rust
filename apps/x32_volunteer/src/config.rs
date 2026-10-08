use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct VolunteerModeConfig {
    pub max_fader_limit_db: Option<f32>,
    pub visible_channels: Option<Vec<u32>>,
    pub visible_dcas: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub volunteer_mode: Option<VolunteerModeConfig>,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> anyhow::Result<AppConfig> {
    let file = File::open(path)?;
    let mut handle = file.take(256 * 1024 + 1);
    let mut content = String::new();
    handle.read_to_string(&mut content)?;
    if content.len() > 256 * 1024 {
        anyhow::bail!("Config file size exceeds 256KB limit");
    }
    let config: AppConfig = serde_json::from_str(&content)?;
    Ok(config)
}
