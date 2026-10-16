//! Startup timing and work that is not needed to render the first window.
use crate::{t_ai, t_sqlite, t_utils};
use std::{
    collections::HashSet,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use tauri::{AppHandle, Emitter, Manager};

pub struct StartupState {
    started: Instant,
    stages: Mutex<HashSet<String>>,
    background_started: AtomicBool,
    ai_status: Mutex<String>,
}

impl Default for StartupState {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            stages: Mutex::new(HashSet::new()),
            background_started: AtomicBool::new(false),
            ai_status: Mutex::new("not-initialized".into()),
        }
    }
}

pub fn mark(app: &AppHandle, stage: &str) {
    let state = app.state::<StartupState>();
    if let Ok(mut stages) = state.stages.lock() {
        if stages.insert(stage.into()) {
            println!(
                "[startup] {}: {} ms",
                stage,
                state.started.elapsed().as_millis()
            );
        }
    }
}

fn initialize_models<T>(
    engine: &Mutex<T>,
    status: &Mutex<String>,
    needs_load: impl FnOnce(&T) -> bool,
    load: impl FnOnce(&mut T) -> Result<(), String>,
) -> Result<(), String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    if !needs_load(&engine) {
        return Ok(());
    }
    *status.lock().map_err(|e| e.to_string())? = "loading".into();
    let result = load(&mut engine);
    *status.lock().map_err(|e| e.to_string())? = match &result {
        Ok(()) => "ready".into(),
        Err(error) => format!("failed: {}", error),
    };
    result
}

// Called from blocking workers. The engine mutex shares initialization with
// early searches/indexing and prevents duplicate model loads.
pub fn ensure_ai(app: &AppHandle, model: Option<i64>) -> Result<(), String> {
    // The oMLX provider computes embeddings over HTTP; keep the bundled ONNX
    // models unloaded while it is active.
    if crate::t_omlx::is_active() {
        return Ok(());
    }
    let state = app.state::<StartupState>();
    let ai = app.state::<t_ai::AiState>();
    initialize_models(
        &ai.0,
        &state.ai_status,
        |engine| !engine.is_loaded() || model.is_some(),
        |engine| {
            mark(app, "ai-load-started");
            // Text-model selection validates dimensions against the loaded vision
            // model and can roll back to the default text model on failure.
            engine.load_models(app)?;
            if let Some(model) = model {
                engine.set_text_model(app, t_ai::ImageSearchTextModel::from_i64(model))?;
            }
            mark(app, "ai-ready");
            Ok(())
        },
    )
}

#[tauri::command]
pub fn record_startup_stage(app: AppHandle, stage: String) {
    if matches!(
        stage.as_str(),
        "app-mounted"
            | "first-content-ready"
            | "language-ready"
            | "library-state-ready"
            | "window-shown"
    ) {
        mark(&app, &stage);
    }
}

#[tauri::command]
pub fn get_ai_initialization_status(app: AppHandle) -> String {
    app.state::<StartupState>()
        .ai_status
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|e| format!("failed: {}", e))
}

#[tauri::command]
pub fn finish_startup(app: AppHandle, model: i64) {
    mark(&app, "first-content-ready");
    if app
        .state::<StartupState>()
        .background_started
        .swap(true, Ordering::SeqCst)
    {
        return;
    }
    let library_id = crate::t_config::current_library_id().ok();
    tauri::async_runtime::spawn_blocking(move || {
        // A missing optional multilingual pack should not prevent vision indexing.
        let model = if model == 1 && !t_ai::AiEngine::is_multilingual_model_available(&app) {
            0
        } else {
            model
        };
        if let Err(error) = ensure_ai(&app, Some(model)) {
            eprintln!("Failed to start AI Engine: {}", error);
            report_ai_dependency_error();
        }
        if let Some(library_id) = library_id {
            let _ = crate::t_cmds::with_current_library(&library_id, || {
                if t_sqlite::is_database_corrupted() {
                    return Ok(());
                }
                let generation = t_utils::album_accessibility_generation();
                let mut albums = t_sqlite::Album::get_all_albums()?;
                t_utils::refresh_all_album_accessibility_if_current(&mut albums, generation);
                if generation == t_utils::album_accessibility_generation() {
                    mark(&app, "album-accessibility-ready");
                    let _ = app.emit(
                        "albums-refreshed",
                        serde_json::json!({
                            "albums": albums, "refreshFolders": false, "libraryId": library_id,
                        }),
                    );
                }
                t_utils::start_folder_mtime_sync(app.clone());
                Ok(())
            });
        }
    });
}

fn report_ai_dependency_error() {
    #[cfg(target_os = "windows")]
    {
        let arch_key = if cfg!(target_arch = "aarch64") {
            r"HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\ARM64"
        } else {
            r"HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64"
        };
        let result = std::process::Command::new("reg")
            .args(["query", arch_key, "/v", "Installed"])
            .stdout(std::process::Stdio::null())
            .status();
        let installed = result.is_ok() && result.unwrap().success();
        if !installed {
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "x64"
            };
            let url = format!("https://aka.ms/vs/17/release/vc_redist.{}.exe", arch);
            let _ = std::process::Command::new("powershell")
                                .args(["-NoProfile", "-Command", &format!(
                                    r#"$wsh = New-Object -ComObject Wscript.Shell; $wsh.Popup('Lap requires the Microsoft Visual C++ Redistributable.`n`nA download page will open in your browser.`nPlease install it, then restart Lap.', 0, 'Lap - Missing Dependency', 0x30); Start-Process '{}'"#,
                                    url
                                )])
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status();
            // Browsing remains usable when optional AI dependencies are missing.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, Barrier},
        thread,
    };

    #[test]
    fn early_ai_requests_share_one_initialization() {
        let engine = Arc::new(Mutex::new(false));
        let status = Arc::new(Mutex::new("not-initialized".into()));
        let loads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(8));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let (engine, status, loads, barrier) = (
                    engine.clone(),
                    status.clone(),
                    loads.clone(),
                    barrier.clone(),
                );
                thread::spawn(move || {
                    barrier.wait();
                    initialize_models(
                        &engine,
                        &status,
                        |loaded| !loaded,
                        |loaded| {
                            assert_eq!(*status.lock().unwrap(), "loading");
                            loads.fetch_add(1, Ordering::SeqCst);
                            *loaded = true;
                            Ok(())
                        },
                    )
                    .unwrap();
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        assert_eq!(loads.load(Ordering::SeqCst), 1);
        assert_eq!(*status.lock().unwrap(), "ready");
    }

    #[test]
    fn failed_initialization_can_retry_without_marking_models_ready() {
        let engine = Mutex::new(false);
        let status = Mutex::new("not-initialized".into());
        assert!(
            initialize_models(
                &engine,
                &status,
                |loaded| !loaded,
                |_| Err("missing model".into())
            )
            .is_err()
        );
        assert_eq!(*status.lock().unwrap(), "failed: missing model");
        assert!(!*engine.lock().unwrap());
        initialize_models(
            &engine,
            &status,
            |loaded| !loaded,
            |loaded| {
                *loaded = true;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(*status.lock().unwrap(), "ready");
    }

    #[test]
    fn background_warmup_runs_once_across_window_remounts() {
        let state = StartupState::default();
        assert!(!state.background_started.swap(true, Ordering::SeqCst));
        assert!(state.background_started.swap(true, Ordering::SeqCst));
    }
}
