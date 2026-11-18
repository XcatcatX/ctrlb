use eframe::egui;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::theme;

// Creates the tray icon. Click on it shows a menu with "Quit".
pub fn create(ctx: &egui::Context) -> Result<TrayIcon, String> {
    let quit_item = MenuItem::new("Quit", true, None);
    let menu = Menu::new();
    menu.append(&quit_item).map_err(|error| error.to_string())?;
    close_app_on_quit(ctx, &quit_item);

    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(theme::APP_NAME)
        .with_icon(build_icon()?)
        .build()
        .map_err(|error| error.to_string())
}

fn close_app_on_quit(ctx: &egui::Context, quit_item: &MenuItem) {
    let quit_id = quit_item.id().clone();
    let ctx = ctx.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == quit_id {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            ctx.request_repaint();
        }
    }));
}

fn build_icon() -> Result<Icon, String> {
    let size = theme::TRAY_ICON_SIZE;
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            pixels.extend_from_slice(&icon_pixel(x, y));
        }
    }
    Icon::from_rgba(pixels, size, size).map_err(|error| error.to_string())
}

// Dark square with a light "paper" and a small "clip" on top
fn icon_pixel(x: u32, y: u32) -> [u8; 4] {
    let is_paper = is_inside(x, y, 9, 7, 23, 26);
    let is_clip = is_inside(x, y, 12, 4, 20, 8);
    if is_paper || is_clip {
        theme::TRAY_ICON_LIGHT
    } else {
        theme::TRAY_ICON_DARK
    }
}

fn is_inside(x: u32, y: u32, left: u32, top: u32, right: u32, bottom: u32) -> bool {
    x >= left && x < right && y >= top && y < bottom
}
