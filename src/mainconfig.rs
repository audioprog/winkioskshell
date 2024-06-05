use confy::ConfyError;
use serde::{Serialize, Deserialize};
use std::{env, path::{Path, PathBuf}};


#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct MainConfig {
    pub client_application: Option<String>,
    pub password: Option<String>,
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            client_application: None,
            password: None,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn load_config(user_name: &str) -> MainConfig {
        let file_path = Self::get_configuration_file_path(user_name);
        confy::load_path(file_path).unwrap_or_default()
    }


    pub fn save_config(config: &MainConfig, user_name: &str) -> Result<(), ConfyError> {
        let file_path = Self::get_configuration_file_path(user_name);
        confy::store_path(file_path, config)
    }


    pub fn get_configuration_file_path(user_name: &str) -> PathBuf {
        let app_name = env!("CARGO_PKG_NAME");
        if let Ok(current_user_path) = confy::get_configuration_file_path(app_name, None) {
            if user_name.is_empty() || user_name.to_lowercase() == whoami::username().to_lowercase() {
                return current_user_path;
            }
            else if let Some(home_dir) = dirs::home_dir() {
                let sub_path = current_user_path.to_str().unwrap().strip_prefix(home_dir.to_str().unwrap()).unwrap();
                let system_drive = env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
                let user_path = Path::new(&system_drive).join("users").join(user_name).join(sub_path);
                return user_path;
            }
        }
        PathBuf::new()
    }


    pub fn set_settings(user_name: &str, application: String, new_password: String) {
        let mut config = Self::load_config(user_name);
        config.client_application = Some(application);
        config.password = Some(new_password);
        let _ = Self::save_config(&config, user_name);
    }
}
