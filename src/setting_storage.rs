use std::convert::{TryFrom, TryInto};
use std::env;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing_unwrap::ResultExt;
use uuid::Uuid;

use tracing::{info, warn};

use crate::types::*;

#[derive(Deserialize, Serialize, Debug)]
struct SettingToml {
    pub uuid: Option<Uuid>,
    pub server: Option<String>,
    pub fullscreen: Option<bool>,
}

#[derive(Serialize)]
pub struct Setting {
    pub uuid: Uuid,
    pub server: String,
    pub fullscreen: bool,
}

impl Setting {
    pub fn load() -> Result<Setting, Box<dyn Error>> {
        let file_path = Setting::file_path()?;

        let mut toml = SettingToml::load(&file_path)?;
        if toml.has_empty_property() {
            warn!("setting lacks some property: {:?}", toml);
            toml.fill_empty_value();
            toml.save(&file_path)?;
        }

        let setting: Setting = toml.try_into().unwrap_or_log();
        Ok(setting)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let file_path = Setting::file_path()?;

        SettingToml::from(self).save(&file_path)
    }

    fn file_path() -> Result<PathBuf, Box<dyn Error>> {
        let mut file_path = PathBuf::new();
        file_path.push(env::current_exe()?);
        file_path = file_path.parent().unwrap_or_log().into();
        file_path.push("blocking-io-settings.toml");
        Ok(file_path)
    }
}

impl SettingToml {
    pub fn empty() -> Self {
        Self {
            uuid: None,
            server: None,
            fullscreen: None,
        }
    }

    #[tracing::instrument("load setting")]
    pub fn load(file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        info!("loading");
        if !file_path.is_file() {
            warn!("setting file not found");
            Ok(SettingToml::empty())
        } else {
            let str = fs::read_to_string(&file_path)?;
            let toml: SettingToml = toml::from_str(&str)?;

            Ok(toml)
        }
    }

    #[tracing::instrument("save setting")]
    pub fn save(&self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        info!("saving");
        let str = toml::to_string(self)?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(file_path)?;

        file.write_all(str.as_bytes())?;

        Ok(())
    }

    fn has_empty_property(&self) -> bool {
        self.uuid.is_none() || self.server.is_none() || self.fullscreen.is_none()
    }

    fn fill_empty_value(&mut self) {
        if self.uuid.is_none() {
            self.uuid = Some(Uuid::new_v4());
        }
        if self.server.is_none() {
            self.server = Some("http://13.114.119.94:3000".to_string());
        }
        if self.fullscreen.is_none() {
            self.fullscreen = Some(false);
        }
        debug_assert!(!self.has_empty_property());
    }
}

impl TryFrom<SettingToml> for Setting {
    type Error = ();

    fn try_from(value: SettingToml) -> Result<Self, Self::Error> {
        if value.has_empty_property() {
            Err(())
        } else {
            Ok(Setting {
                uuid: value.uuid.unwrap_or_log(),
                server: value.server.unwrap_or_log(),
                fullscreen: value.fullscreen.unwrap_or_log(),
            })
        }
    }
}

impl From<&Setting> for SettingToml {
    fn from(setting: &Setting) -> Self {
        SettingToml {
            uuid: Some(setting.uuid),
            server: Some(setting.server.clone()),
            fullscreen: Some(setting.fullscreen),
        }
    }
}
