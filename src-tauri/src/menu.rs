//! Menüleiste (macOS und andere): jeder eigene Eintrag schickt seine ID als Ereignis `menu` an die Oberfläche.

use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Wry};

fn item(app: &AppHandle, id: &str, label: &str, accelerator: Option<&str>) -> tauri::Result<tauri::menu::MenuItem<Wry>> {
    let builder = MenuItemBuilder::with_id(id, label);
    match accelerator {
        Some(a) => builder.accelerator(a).build(app),
        None => builder.build(app),
    }
}

pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let app_menu = SubmenuBuilder::new(app, "Schreibassistent")
        .about(None)
        .separator()
        .item(&item(app, "einstellungen", "Einstellungen …", Some("CmdOrCtrl+,"))?)
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;
    let file = SubmenuBuilder::new(app, "Ablage")
        .item(&item(app, "neu", "Neue Arbeit …", Some("CmdOrCtrl+N"))?)
        .item(&item(app, "oeffnen", "Arbeit öffnen …", Some("CmdOrCtrl+O"))?)
        .separator()
        .item(&item(app, "speichern", "Speichern", Some("CmdOrCtrl+S"))?)
        .item(&item(app, "speichern-unter", "Speichern unter …", Some("CmdOrCtrl+Shift+S"))?)
        .item(&item(app, "pdf", "Als PDF exportieren …", Some("CmdOrCtrl+E"))?)
        .separator()
        .item(&item(app, "finder", "Im Finder zeigen", None)?)
        .item(&item(app, "schliessen", "Arbeit schließen", Some("CmdOrCtrl+Shift+W"))?)
        .build()?;
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
    MenuBuilder::new(app).items(&[&app_menu, &file, &edit, &view, &window, &help]).build()
}
