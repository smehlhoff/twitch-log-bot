use chrono::prelude::*;
use std::fs;
use std::io::{BufWriter, Write};

use crate::lib::error;

#[derive(Copy, Clone, Debug)]
pub struct BotState {
    pub buffer: usize,
    pub paused: bool,
    pub postgres: bool,
    pub uptime: chrono::DateTime<Utc>,
}

impl BotState {
    pub fn new(count: usize, postgres: bool) -> Self {
        let buffer = {
            if count <= 10 {
                100
            } else {
                count * 10
            }
        };

        Self { buffer, paused: false, postgres, uptime: Utc::now() }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BotConfig {
    pub nickname: String,
    pub oauth: String,
    pub server: String,
    pub postgres: String,
    pub admins: Vec<String>,
    pub channels: Vec<String>,
}

impl BotConfig {
    pub fn load() -> Result<Self, error::Error> {
        let file = fs::OpenOptions::new().read(true).open("config.json")?;
        let json: Self = serde_json::from_reader(file)?;

        Ok(json)
    }

    pub fn update(self) -> Result<(), error::Error> {
        fs::remove_file("config.json")?;

        let json = serde_json::to_string_pretty(&self)?;
        let file = fs::OpenOptions::new().create(true).write(true).open("config.json")?;
        let mut file = BufWriter::new(file);

        file.write_all(json.as_bytes())?;

        Ok(())
    }
}
