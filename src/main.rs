#![windows_subsystem = "windows"]

use std::path::PathBuf;

use mainconfig::ConfigManager;

mod mainconfig;
mod win_kiosk_shell;
mod win_kiosk_settings;
mod win_elevation_functions;
mod release;


fn main() {
    let config = ConfigManager::load_config(&"");
    let client_application = config.client_application;
    {
        let app_path = PathBuf::from(if let Some(app) = client_application {app.clone()} else {"".to_owned()});
        let child_option = if app_path.exists() {
            Some(std::process::Command::new(app_path)
                .spawn()
                .expect("The application could not be started."))
        } else {
            None
        };

        match child_option {
            Some(child) => {
                win_kiosk_shell::WinKioskShell::new(child).run();
            },
            None => {
                win_kiosk_settings::WinKioskSettings::new().run();
            }
        }
    }
}
