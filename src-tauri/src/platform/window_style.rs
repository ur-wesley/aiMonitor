use tauri::{AppHandle, Manager, WebviewWindow};

pub fn apply_glass(window: &WebviewWindow) {
    let _ = window.set_shadow(true);

    #[cfg(windows)]
    {
        use tauri::window::{Effect, EffectsBuilder};
        let _ = window.set_effects(
            EffectsBuilder::new()
                .effect(Effect::Acrylic)
                .build(),
        );
    }
}

pub fn apply_glass_for_label(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        apply_glass(&window);
    }
}
