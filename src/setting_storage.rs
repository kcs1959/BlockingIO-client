use std::convert::{TryFrom, TryInto};
use std::env;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct SettingToml {
    pub uuid: Option<Uuid>,
}

#[derive(Serialize)]
pub struct Setting {
    pub uuid: Uuid,
}

impl Setting {
    pub fn load() -> Result<Setting, Box<dyn Error>> {
        let file_path = Setting::file_path()?;
        println!("loading {:?}", &file_path);

        let mut toml = SettingToml::load(&file_path)?;
        if toml.has_empty_property() {
            toml.fill_empty_value();
            toml.save(&file_path)?;
        }

        let setting: Setting = toml.try_into().unwrap();
        Ok(setting)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let file_path = Setting::file_path()?;
        println!("saving {:?}", &file_path);

        SettingToml::from(self).save(&file_path)
    }

    fn file_path() -> Result<PathBuf, Box<dyn Error>> {
        let mut file_path = PathBuf::new();
        file_path.push(env::current_exe()?);
        file_path = file_path.parent().unwrap().into();
        file_path.push("blocking-io-settings.toml");
        Ok(file_path)
    }
}

impl SettingToml {
    pub fn empty() -> Self {
        Self { uuid: None }
    }

    pub fn load(file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        if !file_path.is_file() {
            Ok(SettingToml::empty())
        } else {
            let str = fs::read_to_string(&file_path)?;
            let toml: SettingToml = toml::from_str(&str)?;

            Ok(toml)
        }
    }

    pub fn save(&self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
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
        self.uuid.is_none()
    }

    fn fill_empty_value(&mut self) {
        if self.uuid.is_none() {
            self.uuid = Some(Uuid::new_v4());
        }
    }
}

impl TryFrom<SettingToml> for Setting {
    type Error = ();

    fn try_from(value: SettingToml) -> Result<Self, Self::Error> {
        if value.has_empty_property() {
            Err(())
        } else {
            Ok(Setting {
                uuid: value.uuid.unwrap(),
            })
        }
    }
}

impl From<&Setting> for SettingToml {
    fn from(setting: &Setting) -> Self {
        SettingToml {
            uuid: Some(setting.uuid),
        }
    }
}
