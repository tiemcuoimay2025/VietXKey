// VietFlow Windows - Startup Registry Management
// Licensed under GNU General Public License v3.0 (GPLv3)

use std::ptr::null_mut;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_SET_VALUE, REG_SZ,
};

use crate::tray::to_wide;

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "VietFlow";

fn windows_startup_command(executable: &std::path::Path) -> String {
    format!("\"{}\" --startup", executable.to_string_lossy())
}

/// Manual launches and an explicit elevated restart must present the UI.
/// Login startup alone follows the stored background/window preference.
/// UI-only children are already dispatched before daemon initialization.
pub fn should_show_settings_on_launch(args: &[String], config: &crate::config::AppConfig) -> bool {
    if args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "--settings" | "--gui" | "-g" | "--convert" | "-c"
        )
    }) {
        return false;
    }
    if args.iter().any(|arg| arg == "--elevated") {
        return true;
    }
    if args.iter().any(|arg| arg == "--startup") {
        return config.start_with_windows && config.show_settings_on_startup;
    }
    true
}

pub fn elevated_restart_parameters(config: &std::path::Path) -> String {
    format!("--elevated --config \"{}\"", config.display())
}

pub unsafe fn set_start_with_windows(enable: bool) -> bool {
    let subkey = to_wide(RUN_KEY);
    let mut hkey: HKEY = null_mut();

    let res = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        subkey.as_ptr(),
        0,
        KEY_SET_VALUE,
        &mut hkey,
    );

    if res != ERROR_SUCCESS {
        return false;
    }

    let val_name = to_wide(VALUE_NAME);

    let success = if enable {
        if let Ok(exe_path) = std::env::current_exe() {
            let quoted = windows_startup_command(&exe_path);
            let wide_val = to_wide(&quoted);
            let byte_len = (wide_val.len() * std::mem::size_of::<u16>()) as u32;

            RegSetValueExW(
                hkey,
                val_name.as_ptr(),
                0,
                REG_SZ,
                wide_val.as_ptr() as *const u8,
                byte_len,
            ) == ERROR_SUCCESS
        } else {
            false
        }
    } else {
        RegDeleteValueW(hkey, val_name.as_ptr()) == ERROR_SUCCESS
    };

    RegCloseKey(hkey);
    success
}

#[cfg(test)]
mod startup_settings_tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn startup_command_handles_spaces_and_unicode_and_launches_the_daemon() {
        assert_eq!(
            windows_startup_command(std::path::Path::new(r"H:\Phần mềm\VietXKey\VietXKey.exe")),
            "\"H:\\Phần mềm\\VietXKey\\VietXKey.exe\" --startup"
        );
    }

    #[test]
    fn admin_restart_keeps_the_exact_unicode_config_path() {
        let path = std::path::Path::new(r"H:\Phần mềm\VietXKey\vietxkey_config.json");
        assert_eq!(
            elevated_restart_parameters(path),
            "--elevated --config \"H:\\Phần mềm\\VietXKey\\vietxkey_config.json\""
        );
    }

    #[test]
    fn legacy_startup_config_opens_settings_but_explicit_off_survives_roundtrip() {
        let args = vec!["VietXKey.exe".into(), "--startup".into()];
        let mut config: AppConfig = serde_json::from_str(r#"{"start_with_windows":true}"#).unwrap();
        assert!(config.show_settings_on_startup);
        assert!(should_show_settings_on_launch(&args, &config));
        config.show_settings_on_startup = false;
        let reloaded: AppConfig =
            serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
        assert!(!reloaded.show_settings_on_startup);
        assert!(!should_show_settings_on_launch(&args, &reloaded));
    }

    #[test]
    fn first_manual_launch_and_admin_restart_always_open_even_with_login_opt_out() {
        for start_with_windows in [false, true] {
            for show_settings_on_startup in [false, true] {
                let config = AppConfig {
                    start_with_windows,
                    show_settings_on_startup,
                    ..Default::default()
                };
                for extra in [None, Some("--elevated")] {
                    let mut args = vec!["VietXKey.exe".into()];
                    if let Some(arg) = extra {
                        args.push(arg.into());
                    }
                    assert!(should_show_settings_on_launch(&args, &config));
                }
                let args = vec!["VietXKey.exe".into(), "--startup".into()];
                assert_eq!(
                    should_show_settings_on_launch(&args, &config),
                    start_with_windows && show_settings_on_startup
                );
            }
        }
    }

    #[test]
    fn gui_children_do_not_spawn_a_second_ui_and_admin_takes_precedence_over_login() {
        let config = AppConfig::default();
        for child in ["--settings", "--gui", "-g", "--convert", "-c"] {
            let args = vec!["VietXKey.exe".into(), "--elevated".into(), child.into()];
            assert!(!should_show_settings_on_launch(&args, &config));
        }
        let args = vec![
            "VietXKey.exe".into(),
            "--startup".into(),
            "--elevated".into(),
        ];
        assert!(should_show_settings_on_launch(&args, &config));
    }

    #[test]
    fn stale_windows_startup_entry_does_not_override_the_master_setting() {
        let args = vec!["VietXKey.exe".into(), "--startup".into()];
        assert!(!should_show_settings_on_launch(
            &args,
            &AppConfig::default()
        ));
    }
}

/// Remove all application shortcuts from Start Menu, Desktop, and Startup registry.
/// When `purge_data` is true (full uninstall only), recursively deletes %ProgramData%\VietFlow.
pub unsafe fn clean_all_shortcuts_and_cache(purge_data: bool) {
    // 1. Remove Startup Run entry
    let _ = set_start_with_windows(false);

    // 2. Collect shortcut file targets across user and machine profiles
    let mut candidate_paths: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = std::path::Path::new(&appdata);
        candidate_paths.push(p.join(r"Microsoft\Windows\Start Menu\Programs\VietFlow.lnk"));
        candidate_paths
            .push(p.join(r"Microsoft\Windows\Start Menu\Programs\VietFlow Team\VietFlow.lnk"));
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = std::path::Path::new(&userprofile);
        candidate_paths.push(p.join(r"Desktop\VietFlow.lnk"));
        candidate_paths.push(p.join(r"OneDrive\Desktop\VietFlow.lnk"));
    }
    if let Ok(public) = std::env::var("PUBLIC") {
        let p = std::path::Path::new(&public);
        candidate_paths.push(p.join(r"Desktop\VietFlow.lnk"));
    }
    if let Ok(programdata) = std::env::var("ProgramData") {
        let p = std::path::Path::new(&programdata);
        candidate_paths.push(p.join(r"Microsoft\Windows\Start Menu\Programs\VietFlow.lnk"));
        candidate_paths
            .push(p.join(r"Microsoft\Windows\Start Menu\Programs\VietFlow Team\VietFlow.lnk"));
    }

    for path in &candidate_paths {
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }

    // 3. Remove orphaned Start Menu directories if empty
    if let Ok(appdata) = std::env::var("APPDATA") {
        let team_folder = std::path::Path::new(&appdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\VietFlow Team");
        let _ = std::fs::remove_dir(&team_folder);
    }
    if let Ok(programdata) = std::env::var("ProgramData") {
        let team_folder = std::path::Path::new(&programdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\VietFlow Team");
        let _ = std::fs::remove_dir(&team_folder);
    }

    // 4. If full uninstall, remove the branded shared data folder.
    if purge_data {
        if let Ok(programdata) = std::env::var("ProgramData") {
            let data_dir = std::path::Path::new(&programdata).join(crate::constants::APP_DIR_NAME);
            if data_dir.exists() {
                let _ = std::fs::remove_dir_all(&data_dir);
            }
        }
    }

    // 5. Force Windows Shell icon cache refresh
    windows_sys::Win32::UI::Shell::SHChangeNotify(
        windows_sys::Win32::UI::Shell::SHCNE_ASSOCCHANGED as i32,
        windows_sys::Win32::UI::Shell::SHCNF_IDLIST,
        std::ptr::null(),
        std::ptr::null(),
    );
}
