use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OobeConfig {
    pub locale: Locale,
    pub user: String,
    pub pwd: String,
    pub fullname: Option<String>,
    pub hostname: String,
    pub rtc_as_localtime: bool,
    pub timezone: Timezone,
    pub swapfile: SwapFile,
}

#[derive(Deserialize)]
pub struct SwapFile {
    pub size: f64,
}

#[derive(Deserialize)]
pub struct Locale {
    pub locale: String,
}

#[derive(Deserialize)]
pub struct Timezone {
    pub data: String,
}

pub fn handle_serde_config(s: &str) -> anyhow::Result<OobeConfig> {
    Ok(serde_json::from_str::<OobeConfig>(s)?)
}
