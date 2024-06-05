use std::{env, error::Error, fs, os::windows::process::CommandExt, path::Path, process::{Command, Output, Stdio}, rc::Rc, str::FromStr};
use slint::{self, ComponentHandle, ModelRc, SharedString, VecModel};
use winreg::{enums::KEY_WRITE, RegKey};
use mslnk::ShellLink;
use winapi::um::winbase::CREATE_NO_WINDOW;

use crate::{mainconfig::ConfigManager, win_kiosk_shell::check_for_update};
use crate::win_elevation_functions;


slint::include_modules!();


#[derive(Default)]
pub struct WinKioskSettings {
}

impl WinKioskSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(self) {
        let ui = SettingsWindow::new().unwrap();

        let true_is_admin = win_elevation_functions::WinElevationFunctions::is_admin();
        let is_admin = true;
        ui.set_is_admin(is_admin);
        ui.set_title_text(format!("Kiosk Settings {}", env!("APP_VERSION")).into());

        if is_admin {
            let local_users = if true_is_admin {
                list_local_users().unwrap()
            } else {
                ModelRc::from(Rc::new(VecModel::from(vec![whoami::username().into()])))
            };
            ui.set_users(local_users);
        }
        ui.set_user_info(get_info(&ui.get_selected_user().as_str()).into());

        {
            let config = ConfigManager::load_config(&"");
            if let Some(application_path) = config.client_application.clone() {
                ui.set_client_application(application_path.into());
            }
            if let Some(password) = config.password.clone() {
                ui.set_app_password(password.into());
            }
        }
        
        ui.on_user_selected({
            let ui_handle = ui.as_weak();
            move |selected_user: SharedString| {
                let selected = selected_user.as_str();
                let ui = ui_handle.unwrap();

                let new_info = get_info(selected);

                ui.set_user_info(new_info.into());
            }
        });
        ui.on_search_clicked({
            let ui_handle = ui.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                let file_dialog = rfd::FileDialog::new()
                .add_filter("exe", &["exe"]);
                if let Some(path) = file_dialog.pick_file() {
                    ui.set_client_application(path.display().to_string().into());
                }
            }
        });
        ui.on_request_cancel_close({
            let ui_handle = ui.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                let _ = ui.hide();
            }
        });
        ui.on_request_save_close({
            let ui_handle = ui.as_weak();
            move || {
                let ui = ui_handle.unwrap();

                let app_path = ui.get_client_application().to_string();
                let password = ui.get_app_password().to_string();
                if password.is_empty() {
                    Self::message_box(&ui, "Password must be set.");
                } else {
                    let user_name = ui.get_selected_user();
                    ConfigManager::set_settings(user_name.as_str(), app_path, password);
                    let exe = env::current_exe();
                    let result = write_user_shell(user_name.as_str(), exe.unwrap().to_str().unwrap());
                    if let Err(e) = result {
                        Self::message_box_err(&ui, e);
                    }
                    let _ = ui.hide();
                }
            }
        });

        check_for_update();

        let _ = ui.run();
    }

    fn message_box_err(ui: &SettingsWindow, e: Box<dyn Error>) {
        let err = format!("Error: {}", e);
        Self::message_box(ui, &err);
    }
    
    fn message_box(ui: &SettingsWindow, err: &str) {
        ui.set_dialog_text(err.into());
        ui.invoke_show_message_box();
    }
}

fn get_info(user_name: &str) -> String {
    let current_user_name = whoami::username();
    if user_name.is_empty() || current_user_name == user_name {
        "Cannot set shell, only start in autostart".to_string()
    } else {
        "set shell".to_string()
    }
}

fn write_user_shell(username: &str, client_application_path: &str) -> Result<(), Box<dyn Error>> {
    let system_drive = env::var("SystemDrive")?;
    let user_path = Path::new(&system_drive).join("users").join(username);
    let home_dir = dirs::home_dir().unwrap();

    if username.is_empty() || user_path == home_dir {
        let target = env::current_exe().unwrap();
        let app_name = env!("CARGO_PKG_NAME");
        let mut lnk =  String::from_str(home_dir.to_str().unwrap())?;
        lnk.push_str(r"\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\");
        lnk.push_str(app_name);
        lnk.push_str(".lnk");
        let sl = ShellLink::new(target).unwrap();
        sl.create_lnk(lnk).unwrap();
    } else {
        let file_path = user_path.join(username).join("NTUSER.DAT");
        let reg_key = RegKey::load_app_key(file_path, true)?;
        let subkey_path = r"Software\Microsoft\Windows NT\CurrentVersion\Winlogon";
        let subkey = reg_key.open_subkey_with_flags(subkey_path, KEY_WRITE)?;
        subkey.set_value("Shell", &client_application_path)?;
    }

    Ok(())
}

fn list_local_users() -> Result<ModelRc<SharedString>, std::io::Error> {
    let system_drive = env::var("SystemDrive").unwrap();
    let user_path = Path::new(&system_drive).join("users");

    let local_users: VecModel<SharedString> = VecModel::default();

    if user_path.exists() && user_path.is_dir() {
        for entry in fs::read_dir(user_path)? {
            let dir = entry?;
            if dir.path().is_dir() {
                let ntuser_dat_path = dir.path().join("NTUSER.DAT");
                if ntuser_dat_path.exists() {
                    let user_name = dir.file_name().into_string().unwrap_or_default();
                    if is_local_user(&user_name) {
                        local_users.push(user_name.into());
                    }
                }
            }
        }
    }

    Ok(ModelRc::from(Rc::new(local_users)))
}

fn is_local_user(user_name: &str) -> bool {
    let ps_script = format!("$user = Get-LocalUser -Name '{}'; if($user -ne $null) {{ $true }} else {{ $false }}", user_name);
    let output = run_powershell_script(&ps_script);
    
    match output {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            result.trim() == "True"
        },
        Err(_) => false,
    }
}

fn run_powershell_script(script: &str) -> std::io::Result<Output> {
    let output = Command::new("powershell")
        .args(&["-Command", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW) // Prevents the creation of a window
        .output();

    output
}