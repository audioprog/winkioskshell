use std::env;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{path::Path, thread};
use std::time::Duration;
use slint::{self, ComponentHandle};
use sysinfo::System;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use winapi::um::winuser::{FindWindowA, SetForegroundWindow, GetForegroundWindow};
use std::ffi::CString;
use std::ptr::null_mut;

use crate::mainconfig::ConfigManager;
use crate::{release, win_kiosk_settings};

slint::slint!{
    import { Button, LineEdit } from "std-widgets.slint";

    export component KioskWindow inherits Window {
        in-out property <string> password_text;
        in-out property <string> version;

        callback close();
        callback settings();
        callback start_app();
        callback reboot();

        Rectangle {
            background: black;
            width: 100%;
            height: 100%;

            VerticalLayout {
                width: 10%;
                height: 10%;

                Text {
                    text: version;
                }

                LineEdit {
                    text <=> password_text;
                    input-type: password;
                }

                Button {
                    text: "X";
                    clicked => {root.close();}
                }
                Button {
                    text: "S";
                    clicked => {root.settings();}
                }
                Button {
                    text: "Start";
                    clicked => {root.start_app();}
                }
                Button {
                    text: "🔁";
                    clicked => {root.reboot();}
                }
            }
        }
    }
}

#[derive()]
pub struct WinKioskShell {
    process: Arc<Mutex<std::process::Child>>,
}

impl WinKioskShell {
    pub fn new(winprocess: std::process::Child) -> Self {
        Self { process: Arc::new(Mutex::new(winprocess)) }
    }

    pub fn run(self) {
        let running = Arc::new(AtomicBool::new(true));

        let running_clone = running.clone();
        let process_worker = thread::spawn(move || {
            let client_application = &ConfigManager::load_config(&"").client_application.clone();
            let client_application_name = client_application.as_ref().and_then(|path| {
                Path::new(path).file_name().and_then(|name| name.to_str()).map(|s| s.to_lowercase())
            });

            while running_clone.load(Ordering::SeqCst) {
                let mut system = System::new_all();

                system.refresh_all();
    
                for (_pid, process) in system.processes() {
                    if process.name().to_ascii_lowercase() == "explorer.exe" {
                        process.kill();
                    } else if process.name().to_ascii_lowercase() == "msedge.exe" {
                        process.kill();
                    } else if let Some(process_name_lower) = client_application_name.as_deref() {
                        let _ = set_focus_to_application(process_name_lower);
                    }
                }
    
                thread::sleep(Duration::from_secs(1));
            }
        });

        let window = KioskWindow::new().unwrap();
        window.set_version(env!("APP_VERSION").into());
        window.on_close({
            let ui_handle = window.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                let password = ui.get_password_text().to_string();
                let check = ConfigManager::load_config(&"").password.clone();
                if check.is_none() || check.unwrap() == password {
                    let _ = ui.hide();
                }
            }
        });
        window.on_settings({
            let ui_handle = window.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                let password = ui.get_password_text().to_string();
                let check = ConfigManager::load_config(&"").password.clone();
                if check.is_none() || check.unwrap() == password {
                    win_kiosk_settings::WinKioskSettings::new().run();
                    let _ = ui.hide();
                }
            }
        });
        window.on_start_app({
            let process_clone = self.process.clone();
            let running_clone = running.clone();
            move || {
                if !running_clone.load(Ordering::SeqCst) {
                    return;
                }

                let config = ConfigManager::load_config(&"");
                let client_application = config.client_application;
                let app_path = PathBuf::from(if let Some(app) = client_application {app.clone()} else {"".to_owned()});
                let mut proc = process_clone.lock().unwrap();
                *proc = std::process::Command::new(app_path).spawn().unwrap();

                check_for_update();
            }
        });
        window.on_reboot({
            move || {
                match std::process::Command::new("shutdown").args(&["/r", "/t", "0"]).spawn() {
                    Ok(_) => println!("Windows reboot."),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        });
        window.window().set_fullscreen(true);
        let _ = window.show();
        let _ = window.run();

        running.store(false, Ordering::SeqCst);

        process_worker.join().unwrap();

        let mut proc = self.process.lock().unwrap();
        let _ = proc.wait();
    }
}

fn set_focus_to_application(application_name: &str) -> Result<(), String> {
    unsafe {
        let window_name = CString::new(application_name).unwrap();
        let target_hwnd = FindWindowA(null_mut(), window_name.as_ptr());

        if target_hwnd.is_null() {
            return Err("Window not found.".to_owned());
        }

        let foreground_hwnd = GetForegroundWindow();

        if target_hwnd != foreground_hwnd {
            SetForegroundWindow(target_hwnd);
        }
        return Ok(());
    }
}

pub fn check_for_update() {
    if let Ok(release) = release::get_latest_release("audioprog/winkioskshell") {
        if let Ok(is_newer) = release::is_update_available(&release) {
            if is_newer {
                if let Some(asset) = release.assets.first() {
                    let url_string: &String = &asset.browser_download_url;
                    if let Ok(mut exe) = env::current_exe() {
                        exe.pop();
                        let _ = release::download_latest_release(&url_string, exe.as_path());
                    }
                }
            }
        }
    }
}

