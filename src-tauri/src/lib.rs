use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size,
    WebviewBuilder, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use url::Url;

static CLICK_THROUGH: AtomicBool = AtomicBool::new(false);
static BAR_VISIBLE: AtomicBool = AtomicBool::new(true);
const BAR_HEIGHT: f64 = 44.0;

// macOSにおいて、Spaces（仮想デスクトップ）切り替えや全画面アプリ上でもオーバーレイウィンドウを表示し続けるための設定
#[cfg(target_os = "macos")]
fn setup_macos_spaces_behavior(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSMainMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};
    if let Ok(ns_win_ptr) = window.ns_window() {
        unsafe {
            let ns_win = &*(ns_win_ptr as *const NSWindow);
            let existing = ns_win.collectionBehavior();
            // CanJoinAllSpaces: すべてのSpaceで表示
            // FullScreenAuxiliary: 全画面表示中の他アプリ（フルスクリーンSpace）の上にもオーバーレイ表示
            // Stationary: ExposeやSpaces移動時にも位置を固定
            // IgnoresCycle: ウィンドウ切り替えサイクルから除外
            ns_win.setCollectionBehavior(
                existing
                    | NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::IgnoresCycle,
            );
            // フルスクリーンアプリの上に乗るようにウィンドウレベルを引き上げる
            ns_win.setLevel(NSMainMenuWindowLevel);
            ns_win.orderFrontRegardless();
        }
    }
}

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
        #[cfg(target_os = "macos")]
        if enabled {
            setup_macos_spaces_behavior(&window);
        }
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

            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            
            // メインウィンドウ生成 (Accessory設定後に生成することでフルスクリーンSpaceオーバーレイを有効化)
            let window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title("Overlay Player")
            .inner_size(720.0, 450.0)
            .min_inner_size(320.0, 200.0)
            .resizable(true)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .shadow(true)
            .build()
            .map_err(|e| e.to_string())?;

            // macOSでSpaces（仮想デスクトップ）や全画面アプリを跨いでも表示されるように設定
            #[cfg(target_os = "macos")]
            setup_macos_spaces_behavior(&window);

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
            minimize_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
