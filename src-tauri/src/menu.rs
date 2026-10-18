//! Menüleiste (macOS mit App-Menü; Windows und Linux mit „Datei“ samt „Beenden“): jeder eigene Eintrag schickt seine ID als Ereignis `menu` an die Oberfläche.

use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Wry};

fn item(app: &AppHandle, id: &str, label: &str, accelerator: Option<&str>) -> tauri::Result<tauri::menu::MenuItem<Wry>> {
    let builder = MenuItemBuilder::with_id(id, label);
    match accelerator {
        Some(a) => builder.accelerator(a).build(app),
        None => builder.build(app),
    }
}

/// Name des Dateimanagers im Menü.
fn file_manager() -> &'static str {
    if cfg!(target_os = "macos") { "Im Finder zeigen" } else { "Im Dateimanager zeigen" }
}

pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let settings = item(app, "einstellungen", "Einstellungen …", Some("CmdOrCtrl+,"))?;
    let mut file = SubmenuBuilder::new(app, if cfg!(target_os = "macos") { "Ablage" } else { "Datei" })
        .item(&item(app, "neu", "Neue Arbeit …", Some("CmdOrCtrl+N"))?)
        .item(&item(app, "oeffnen", "Arbeit öffnen …", Some("CmdOrCtrl+O"))?)
        .separator()
        .item(&item(app, "speichern", "Speichern", Some("CmdOrCtrl+S"))?)
        .item(&item(app, "speichern-unter", "Speichern unter …", Some("CmdOrCtrl+Shift+S"))?)
        .item(&item(app, "pdf", "Als PDF exportieren …", Some("CmdOrCtrl+E"))?)
        .separator()
        .item(&item(app, "finder", file_manager(), None)?)
        .item(&item(app, "schliessen", "Arbeit schließen", Some("CmdOrCtrl+Shift+W"))?);
    if !cfg!(target_os = "macos") {
        // Ohne App-Menü: Einstellungen und Beenden gehören hierher.
        file = file.separator().item(&settings).separator().quit();
    }
    let file = file.build()?;
    let edit = SubmenuBuilder::new(app, "Bearbeiten")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let view = SubmenuBuilder::new(app, "Darstellung")
        .item(&item(app, "ansicht-seite", "Seite", None)?)
        .item(&item(app, "ansicht-quelltext", "Quelltext", None)?)
        .separator()
        .item(&item(app, "hell", "Hell", None)?)
        .item(&item(app, "dunkel", "Dunkel", None)?)
        .item(&item(app, "system", "System", None)?)
        .separator()
        .item(&item(app, "links", "Linke Spalte ein-/ausblenden", Some("CmdOrCtrl+Alt+Left"))?)
        .item(&item(app, "rechts", "Rechte Spalte ein-/ausblenden", Some("CmdOrCtrl+Alt+Right"))?)
        .separator()
        .item(&item(app, "palette", "Befehle …", None)?)
        .fullscreen()
        .build()?;
    let window = SubmenuBuilder::new(app, "Fenster").minimize().maximize().close_window().build()?;
    let help = SubmenuBuilder::new(app, "Hilfe").item(&item(app, "tastenkuerzel", "Tastenkürzel", None)?).build()?;
    #[cfg(target_os = "macos")]
    {
        let app_menu = SubmenuBuilder::new(app, "Schreibassistent")
            .about(None)
            .separator()
            .item(&settings)
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?;
        return MenuBuilder::new(app).items(&[&app_menu, &file, &edit, &view, &window, &help]).build();
    }
    #[cfg(not(target_os = "macos"))]
    MenuBuilder::new(app).items(&[&file, &edit, &view, &window, &help]).build()
}
