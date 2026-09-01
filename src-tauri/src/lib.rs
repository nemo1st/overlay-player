use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size,
    WebviewBuilder, WebviewUrl,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use url::Url;

static CLICK_THROUGH: AtomicBool = AtomicBool::new(false);
static BAR_VISIBLE: AtomicBool = AtomicBool::new(true);
const BAR_HEIGHT: f64 = 44.0;

#[derive(Default)]
pub struct AppState {
    pub current_url: Mutex<String>,
}

#[tauri::command]
fn set_url(app: AppHandle, url: String) -> Result<(), String> {
    let parsed_url = Url::parse(&url).map_err(|e| e.to_string())?;
    if let Some(content_webview) = app.get_webview("content") {
        content_webview
            .navigate(parsed_url)
            .map_err(|e| e.to_string())?;
    }
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut current) = state.current_url.lock() {
            *current = url;
        }
    }
    Ok(())
}

#[tauri::command]
fn get_current_url(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let current = state.current_url.lock().map_err(|e| e.to_string())?;
    Ok(current.clone())
}

#[tauri::command]
fn set_always_on_top(app: AppHandle, enabled: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(enabled).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn set_click_through(app: AppHandle, enabled: bool) -> Result<bool, String> {
    CLICK_THROUGH.store(enabled, Ordering::SeqCst);
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_ignore_cursor_events(enabled)
            .map_err(|e| e.to_string())?;
    }
    let _ = app.emit("click-through-changed", enabled);
    Ok(enabled)
}

#[tauri::command]
fn toggle_click_through(app: AppHandle) -> Result<bool, String> {
    let new_state = !CLICK_THROUGH.load(Ordering::SeqCst);
    set_click_through(app, new_state)
}

#[tauri::command]
fn set_preset_size(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let is_bar_visible = BAR_VISIBLE.load(Ordering::SeqCst);
        let total_height = if is_bar_visible { height + BAR_HEIGHT } else { height };
        window
            .set_size(tauri::LogicalSize::new(width, total_height))
            .map_err(|e| e.to_string())?;
    }
    update_webview_bounds(&app)?;
    Ok(())
}

#[tauri::command]
fn nav_back(app: AppHandle) -> Result<(), String> {
    if let Some(content) = app.get_webview("content") {
        let _ = content.eval("window.history.back()");
    }
    Ok(())
}

#[tauri::command]
fn nav_forward(app: AppHandle) -> Result<(), String> {
    if let Some(content) = app.get_webview("content") {
        let _ = content.eval("window.history.forward()");
    }
    Ok(())
}

#[tauri::command]
fn nav_reload(app: AppHandle) -> Result<(), String> {
    if let Some(content) = app.get_webview("content") {
        let _ = content.eval("window.location.reload()");
    }
    Ok(())
}

#[tauri::command]
fn optimize_prime_video(app: AppHandle) -> Result<(), String> {
    // Prime Videoのヘッダーやフッターをミニマル化するスクリプト注入
    if let Some(content) = app.get_webview("content") {
        let js = r#"
            (function() {
                const styleId = 'overlay-custom-style';
                let style = document.getElementById(styleId);
                if (!style) {
                    style = document.createElement('style');
                    style.id = styleId;
                    style.innerHTML = `
                        /* Prime Video 上部ナビゲーションのコンパクト化 */
                        #pv-nav-container, header[role="banner"], .DVWebNode-topbar {
                            opacity: 0.1 !important;
                            transition: opacity 0.3s ease !important;
                        }
                        #pv-nav-container:hover, header[role="banner"]:hover, .DVWebNode-topbar:hover {
                            opacity: 1 !important;
                        }
                        /* 背景を黒で統一 */
                        body { background-color: #000 !important; }
                    `;
                    document.head.appendChild(style);
                }
            })();
        "#;
        let _ = content.eval(js);
    }
    Ok(())
}

#[tauri::command]
fn set_bar_visible(app: AppHandle, visible: bool) -> Result<bool, String> {
    BAR_VISIBLE.store(visible, Ordering::SeqCst);
    update_webview_bounds(&app)?;
    let _ = app.emit("bar-visibility-changed", visible);
    Ok(visible)
}

#[tauri::command]
fn toggle_bar_visible(app: AppHandle) -> Result<bool, String> {
    let new_state = !BAR_VISIBLE.load(Ordering::SeqCst);
    set_bar_visible(app, new_state)
}

#[tauri::command]
fn sync_bounds(app: AppHandle) -> Result<(), String> {
    update_webview_bounds(&app)
}

#[tauri::command]
fn close_app(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
    Ok(())
}

#[tauri::command]
fn move_window_by(app: AppHandle, delta_x: f64, delta_y: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let current_pos = window.outer_position().map_err(|e| e.to_string())?;
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let new_x = current_pos.x + (delta_x * scale_factor) as i32;
        let new_y = current_pos.y + (delta_y * scale_factor) as i32;
        window
            .set_position(Position::Physical(PhysicalPosition::new(new_x, new_y)))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn start_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    let _ = window.start_dragging();
    Ok(())
}

#[tauri::command]
fn minimize_app(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
    Ok(())
}

fn update_webview_bounds(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let size = window.inner_size().map_err(|e| e.to_string())?;
        let is_bar_visible = BAR_VISIBLE.load(Ordering::SeqCst);
        let bar_h = if is_bar_visible { BAR_HEIGHT } else { 0.0 };
        let scale_factor = window.scale_factor().unwrap_or(1.0);

        // UI コントローラー用Webview (main)
        if let Some(ui_webview) = app.get_webview("main") {
            let ui_height = if is_bar_visible {
                BAR_HEIGHT
            } else {
                // 非表示時もホバー検知用に上部6pxだけ残すか0にする
                6.0
            };
            let ui_phys_h = (ui_height * scale_factor) as u32;
            let _ = ui_webview.set_bounds(tauri::Rect {
                position: Position::Physical(PhysicalPosition::new(0, 0)),
                size: Size::Physical(PhysicalSize::new(size.width, ui_phys_h)),
            });
        }

        // コンテンツ用Webview (content)
        if let Some(content_webview) = app.get_webview("content") {
            let y_offset = (bar_h * scale_factor) as i32;
            let content_height = size.height.saturating_sub((bar_h * scale_factor) as u32);

            let _ = content_webview.set_bounds(tauri::Rect {
                position: Position::Physical(PhysicalPosition::new(0, y_offset)),
                size: Size::Physical(PhysicalSize::new(size.width, content_height)),
            });
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState {
            current_url: Mutex::new("https://www.amazon.co.jp/gp/video/storefront".into()),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            
            // メインウィンドウ取得
            let window = app.get_webview_window("main").expect("Main window not found");

            // 初期サイズ
            let size = window.inner_size().unwrap_or(PhysicalSize::new(720, 450));
            let scale_factor = window.scale_factor().unwrap_or(1.0);
            let bar_phys_h = (BAR_HEIGHT * scale_factor) as u32;

            // UI用 Webviewの bounds を上部バーに設定
            if let Some(ui_webview) = app.get_webview("main") {
                let _ = ui_webview.set_bounds(tauri::Rect {
                    position: Position::Physical(PhysicalPosition::new(0, 0)),
                    size: Size::Physical(PhysicalSize::new(size.width, bar_phys_h)),
                });
            }

            // コンテンツ用 Webview を作成 (Safari User-Agent を設定してDRM再生を有効化)
            let prime_video_url = "https://www.amazon.co.jp/gp/video/storefront";
            let safari_user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";
            let builder = WebviewBuilder::new("content", WebviewUrl::External(prime_video_url.parse().unwrap()))
                .user_agent(safari_user_agent)
                .auto_resize();

            let _ = window.as_ref().window().add_child(
                builder,
                Position::Physical(PhysicalPosition::new(0, bar_phys_h as i32)),
                Size::Physical(PhysicalSize::new(size.width, size.height.saturating_sub(bar_phys_h))),
            );

            // ウィンドウのリサイズを監視してWebviewのboundsを追従
            let handle_clone = handle.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(_) = event {
                    let _ = update_webview_bounds(&handle_clone);
                }
            });

            // グローバルショートカット登録 (Cmd+Shift+X でクリックスルー切り替え)
            #[cfg(desktop)]
            {
                let shortcut_str = "CommandOrControl+Shift+X";
                if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                    let handle_shortcut = handle.clone();
                    let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                        if event.state() == ShortcutState::Pressed {
                            let _ = toggle_click_through(handle_shortcut.clone());
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_url,
            get_current_url,
            set_always_on_top,
            set_click_through,
            toggle_click_through,
            set_preset_size,
            optimize_prime_video,
            nav_back,
            nav_forward,
            nav_reload,
            set_bar_visible,
            toggle_bar_visible,
            sync_bounds,
            close_app,
            minimize_app,
            start_drag,
            move_window_by
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
