// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::State;

mod sensor;

/// Holds handles to all background child processes so we can kill them on exit.
struct AppProcesses {
    agent: Option<Child>,
    audio: Option<Child>,
    vision: Option<Child>,
}

impl AppProcesses {
    fn new() -> Self {
        Self { agent: None, audio: None, vision: None }
    }

    fn kill_all(&mut self) {
        for (name, child) in [
            ("agent", &mut self.agent),
            ("audio", &mut self.audio),
            ("vision", &mut self.vision),
        ] {
            if let Some(mut proc) = child.take() {
                let pid = proc.id();
                let _ = proc.kill();
                println!("🛑 Killed {} (pid {})", name, pid);
            }
        }
        // Belt-and-suspenders: kill by port to catch any orphans
        let _ = Command::new("sh")
            .args(["-c", "lsof -ti :44444,8000,8001 | xargs kill -9 2>/dev/null || true"])
            .status();
    }
}

type ProcessState = Arc<Mutex<AppProcesses>>;

/// Resolve a path relative to the project root (Momentum AI/).
/// In dev, `tauri dev` is run from ui/, so cwd().parent() = project root.
fn project_path(rel: &str) -> PathBuf {
    // Strategy 1: current_dir in dev is ui/ → go up one level
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(parent) = cwd.parent() {
            let candidate = parent.join(rel);
            if candidate.exists() {
                return candidate;
            }
        }
        // Maybe already at project root
        let candidate = cwd.join(rel);
        if candidate.exists() {
            return candidate;
        }
    }

    // Strategy 2: walk exe path ancestors (dev exe is at ui/src-tauri/target/debug/binary)
    let exe = std::env::current_exe().unwrap_or_default();
    for depth in [5usize, 4, 6, 3, 7] {
        if let Some(base) = exe.ancestors().nth(depth) {
            let candidate = base.join(rel);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    // Strategy 3: hardcoded fallback
    PathBuf::from("/Users/sanjeevn/Downloads/Momentum AI").join(rel)
}

fn find_venv_binary(dir: &PathBuf, name: &str) -> PathBuf {
    #[cfg(windows)]
    let venv_bin = dir.join("venv").join("Scripts").join(format!("{}.exe", name));
    #[cfg(not(windows))]
    let venv_bin = dir.join("venv/bin").join(name);

    if venv_bin.exists() {
        venv_bin
    } else {
        PathBuf::from(name)
    }
}

/// Kill anything already running on our ports before we start
fn clear_ports() {
    let _ = Command::new("sh")
        .args(["-c", "lsof -ti :44444,8000,8001 | xargs kill -9 2>/dev/null || true"])
        .status();
    std::thread::sleep(std::time::Duration::from_millis(600));
}

fn launch_processes() -> AppProcesses {
    clear_ports();
    let mut procs = AppProcesses::new();

    // ── 1. Momentum Rust Agent (port 44444) ───────────────────────────────
    let agent_dir = project_path("momentum-agent");
    #[cfg(target_os = "windows")]
    let agent_bin = agent_dir.join("target").join("release").join("momentum-agent.exe");
    #[cfg(not(target_os = "windows"))]
    let agent_bin = agent_dir.join("target").join("release").join("momentum-agent");
    println!("🚀 Launching Agent: {:?}", agent_bin);
    match Command::new(&agent_bin).current_dir(&agent_dir).spawn() {
        Ok(child) => { println!("✅ Agent pid {}", child.id()); procs.agent = Some(child); }
        Err(e) => eprintln!("❌ Agent failed: {} (path: {:?})", e, agent_bin),
    }

    // ── 2. Momentum Audio (port 8000) ─────────────────────────────────────
    let audio_dir = project_path("momentum-audio");
    let uvicorn = find_venv_binary(&audio_dir, "uvicorn");
    println!("🎤 Launching Audio: {:?}", uvicorn);
    match Command::new(&uvicorn)
        .args(["server:app", "--host", "127.0.0.1", "--port", "8000"])
        .current_dir(&audio_dir)
        .spawn()
    {
        Ok(child) => { println!("✅ Audio pid {}", child.id()); procs.audio = Some(child); }
        Err(e) => eprintln!("❌ Audio failed: {} (path: {:?})", e, uvicorn),
    }

    // ── 3. Momentum Vision (port 8001) ────────────────────────────────────
    let vision_dir = project_path("momentum-vision");
    let python = find_venv_binary(&vision_dir, "python");
    println!("👁️  Launching Vision: {:?}", python);
    match Command::new(&python)
        .args(["engine.py"])
        .current_dir(&vision_dir)
        .spawn()
    {
        Ok(child) => { println!("✅ Vision pid {}", child.id()); procs.vision = Some(child); }
        Err(e) => eprintln!("❌ Vision failed: {} (path: {:?})", e, python),
    }

    procs
}

#[tauri::command]
fn get_status(state: State<ProcessState>) -> serde_json::Value {
    let procs = state.lock().unwrap();
    serde_json::json!({
        "agent": procs.agent.is_some(),
        "audio": procs.audio.is_some(),
        "vision": procs.vision.is_some(),
    })
}

#[tauri::command]
fn hide_window_now(window: tauri::Window) {
    let _ = window.hide();
}

#[tauri::command]
fn show_window_now(window: tauri::Window) {
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn run() {
    let process_state: ProcessState = Arc::new(Mutex::new(AppProcesses::new()));
    let process_state_for_exit = process_state.clone();

    // Launch all backend processes before window opens
    {
        let mut procs = process_state.lock().unwrap();
        *procs = launch_processes();
    }

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            use tauri::{Manager, Emitter};
            use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
            use tauri::menu::{Menu, MenuItem};

            if let Some(window) = app.get_webview_window("main") {
                // Start hidden!
                let _ = window.hide();
                
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let screen_size = monitor.size();
                    let window_size = window.outer_size().unwrap_or_default();
                    #[cfg(target_os = "macos")]
                    {
                        let x = (screen_size.width as i32 - window_size.width as i32) / 2;
                        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y: 40 }));
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let x = (screen_size.width as i32 - window_size.width as i32) / 2;
                        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y: 0 }));
                    }
                }
            }

            // Build Tray Icon
            let toggle_i = MenuItem::with_id(app, "toggle", "Show/Hide Momentum", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "toggle" => {
                        if let Some(window) = app.get_webview_window("main") {
                            // If it's completely hidden, we need to show it so React can run the animation
                            if !window.is_visible().unwrap_or(false) {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            let _ = window.emit("visibility-toggle", "tray");
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .manage(process_state)
        .invoke_handler(tauri::generate_handler![get_status, hide_window_now, show_window_now])
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                println!("🛑 Window closed — killing all backends...");
                if let Ok(mut procs) = process_state_for_exit.lock() {
                    procs.kill_all();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Momentum AI failed to start");
}
