use tauri::menu::MenuEvent;
#[cfg(target_os = "macos")]
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Runtime};
#[cfg(target_os = "macos")]
use tauri::{Emitter, Manager};

#[cfg(target_os = "macos")]
pub fn install_app_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let preferences_item = MenuItemBuilder::with_id("preferences", "Preferences…")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let about_item =
        MenuItemBuilder::with_id("about", format!("About {}", app.package_info().name))
            .build(app)?;
    let home_item = MenuItemBuilder::with_id("help_homepage", "Lap Home").build(app)?;
    let release_notes_item =
        MenuItemBuilder::with_id("help_release_notes", "Release Notes").build(app)?;
    let readme_item = MenuItemBuilder::with_id("help_readme", "README").build(app)?;
    let privacy_item = MenuItemBuilder::with_id("help_privacy", "Privacy").build(app)?;
    let report_issue_item =
        MenuItemBuilder::with_id("help_report_issue", "Report an Issue").build(app)?;
    // The predefined select-all item only reaches text-edit responders, so it
    // silently did nothing while photos were shown. Route a custom item to the
    // photo selection instead. The accelerator uses the character the active
    // layout produces, so it stays Cmd-A on AZERTY as well.
    let select_all_item = MenuItemBuilder::with_id("select_all", "Select All")
        .accelerator("CmdOrCtrl+A")
        .build(app)?;

    let app_submenu = SubmenuBuilder::new(app, app.package_info().name.clone())
        .item(&about_item)
        .separator()
        .item(&preferences_item)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;
    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .item(&select_all_item)
        .build()?;
    let window_submenu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize_with_text("Zoom")
        .separator()
        .close_window()
        .build()?;
    let help_submenu = SubmenuBuilder::new(app, "Help")
        .item(&home_item)
        .item(&release_notes_item)
        .item(&readme_item)
        .item(&privacy_item)
        .separator()
        .item(&report_issue_item)
        .build()?;
    let menu = MenuBuilder::new(app)
        .item(&app_submenu)
        .item(&edit_submenu)
        .item(&window_submenu)
        .item(&help_submenu)
        .build()?;

    app.set_menu(menu)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn install_app_menu<R: Runtime>(_app: &AppHandle<R>) -> tauri::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    match event.id().0.as_ref() {
        "select_all" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("menu-select-all", ());
            }
        }
        "about" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("app-open-about", ());
            }
        }
        "preferences" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("app-open-preferences", ());
            }
        }
        "help_homepage" => {
            let _ = opener::open(
                option_env!("CARGO_PKG_HOMEPAGE").unwrap_or("https://julyx10.github.io/lap"),
            );
        }
        "help_release_notes" => {
            let _ = opener::open(format!(
                "{}/guide/release-notes/v{}.html",
                option_env!("CARGO_PKG_HOMEPAGE").unwrap_or("https://julyx10.github.io/lap"),
                env!("CARGO_PKG_VERSION")
            ));
        }
        "help_readme" => {
            let _ = opener::open(format!(
                "{}/blob/main/README.md",
                option_env!("CARGO_PKG_REPOSITORY").unwrap_or("https://github.com/julyx10/lap")
            ));
        }
        "help_privacy" => {
            let _ = opener::open(format!(
                "{}/blob/main/PRIVACY.md",
                option_env!("CARGO_PKG_REPOSITORY").unwrap_or("https://github.com/julyx10/lap")
            ));
        }
        "help_report_issue" => {
            let _ = opener::open(format!(
                "{}/issues",
                option_env!("CARGO_PKG_REPOSITORY").unwrap_or("https://github.com/julyx10/lap")
            ));
        }
        _ => {}
    }
}

#[cfg(not(target_os = "macos"))]
pub fn handle_menu_event<R: Runtime>(_app: &AppHandle<R>, _event: MenuEvent) {}
