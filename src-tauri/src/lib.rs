use tauri::{menu::{MenuBuilder, MenuItemBuilder}, tray::TrayIconBuilder, Manager, State, WindowEvent};
use fs_extra::dir::CopyOptions;
use std::sync::Mutex;
use winreg::enums::*;
use winreg::RegKey;
use tauri::AppHandle;

// State management for the list of files to copy
pub struct AppState {
    pub copy_list: Mutex<Vec<String>>,
}

#[tauri::command]
fn add_to_copy_list(path: String, state: State<'_, AppState>) {
    state.copy_list.lock().unwrap().push(path);
}

#[tauri::command]
fn get_copy_list(state: State<'_, AppState>) -> Vec<String> {
    state.copy_list.lock().unwrap().clone()
}

#[tauri::command]
fn clear_copy_list(state: State<'_, AppState>) {
    state.copy_list.lock().unwrap().clear();
}

#[tauri::command]
fn remove_from_copy_list(path: String, state: State<'_, AppState>) {
    state.copy_list.lock().unwrap().retain(|x| x != &path);
}

#[tauri::command]
fn copy_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    if sources.is_empty() {
        return Err("The copy list is empty.".to_string());
    }
    let options = CopyOptions::new(); // This can be configured, e.g., options.overwrite = true;
    match fs_extra::copy_items(&sources, &destination, &options) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// Function to handle the context menu command logic
fn handle_context_menu_command(app_handle: &AppHandle, args: &[String]) {
    if args.len() > 1 {
        let state = app_handle.state::<AppState>();
        let window = app_handle.get_webview_window("main").unwrap();

        match args[1].as_str() {
            "--copy" => {
                if let Some(path) = args.get(2) {
                    state.copy_list.lock().unwrap().push(path.clone());
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            "--paste" => {
                if let Some(destination) = args.get(2) {
                    let sources = state.copy_list.lock().unwrap().clone();
                    if !sources.is_empty() {
                        match copy_items(sources, destination.clone()) {
                            Ok(_) => {
                                println!("Successfully pasted files.");
                                state.copy_list.lock().unwrap().clear();
                            }
                            Err(e) => {
                                println!("Error pasting files: {}", e);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn register_context_menu() -> std::io::Result<()> {
    // Use HKEY_CURRENT_USER for per-user registration, which does not require admin privileges.
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (shell_key, _) = hkcu.create_subkey("Software\\Classes")?;  // This is the equivalent of HKCR for HKCU

    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_str().unwrap();

    // Super Copy for files
    let (key, _) = shell_key.create_subkey("*\\shell\\SuperCopy")?;
    key.set_value("", &"Super Copy")?;
    // Set icon to a known file path that will persist, but for HKCU, the icon may still be an issue.
    // The main fix is the registry location.
    key.set_value("Icon", &format!("{},0", exe_path_str))?; 
    let (command_key, _) = key.create_subkey("command")?;
    command_key.set_value("", &format!("\"{}\" \"--copy\" \"%1\"", exe_path_str))?;

    // Super Copy for folders
    let (key, _) = shell_key.create_subkey("Directory\\shell\\SuperCopy")?;
    key.set_value("", &"Super Copy")?;
    key.set_value("Icon", &format!("{},0", exe_path_str))?;
    let (command_key, _) = key.create_subkey("command")?;
    command_key.set_value("", &format!("\"{}\" \"--copy\" \"%1\"", exe_path_str))?;

    // Super Paste for folder backgrounds
    let (key, _) = shell_key.create_subkey("Directory\\Background\\shell\\SuperPaste")?;
    key.set_value("", &"Super Paste")?;
    key.set_value("Icon", &format!("{},0", exe_path_str))?;
    let (command_key, _) = key.create_subkey("command")?;
    command_key.set_value("", &format!("\"{}\" \"--paste\" \"%V\"", exe_path_str))?;

    Ok(())
}

fn unregister_context_menu() -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let shell_key = hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS)?;
    
    // Note: Deleting subkeys from HKCU\Software\Classes is equivalent to deleting from HKCR for the current user.
    // The winreg library should handle this correctly.
    shell_key.delete_subkey_all("*\\shell\\SuperCopy")?;
    shell_key.delete_subkey_all("Directory\\shell\\SuperCopy")?;
    shell_key.delete_subkey_all("Directory\\Background\\shell\\SuperPaste")?;
    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState { copy_list: Mutex::new(Vec::new()) })
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window.hide().unwrap();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            copy_items,
            add_to_copy_list,
            get_copy_list,
            clear_copy_list,
            remove_from_copy_list
        ])
        .setup(|app| {
            register_context_menu().expect("Failed to register context menu");
            
            let handle = app.handle();
            let menu = MenuBuilder::new(handle)
                .item(&MenuItemBuilder::new("Show").id("show").build(app.handle())?)
                .item(&MenuItemBuilder::new("Hide").id("hide").build(app.handle())?)
                .item(&MenuItemBuilder::new("Quit").id("quit").build(app.handle())?)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    let window = app.get_webview_window("main").unwrap();
                    match event.id.as_ref() {
                        "quit" => {
                            unregister_context_menu().expect("Failed to unregister context menu");
                            app.exit(0);
                        }
                        "show" => {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                        "hide" => {
                            window.hide().unwrap();
                        }
                        _ => {}
                    }
                })
                .build(app.handle())?;

            // Check for command line arguments and handle them
            let args: Vec<String> = std::env::args().collect();
            handle_context_menu_command(app.handle(), &args);

            Ok(())
        })
        // Add the single instance plugin
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // New instance launched, send the arguments to the running instance
            println!("New instance launched with args: {:?}", args);
            
            // Handle the context menu command logic in the running instance
            handle_context_menu_command(app, &args);
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
