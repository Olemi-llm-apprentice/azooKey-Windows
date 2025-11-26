mod ipc;

use serde::{Deserialize, Serialize};
use shared::{AppConfig, UserDictEntry, UserDictionary};
use std::{path::PathBuf, sync::Mutex};

#[derive(Debug)]
pub struct AppState {
    settings: Mutex<AppConfig>,
    ipc: ipc::IPCService,
}

impl AppState {
    fn new() -> Self {
        AppState {
            settings: Mutex::new(AppConfig::new()),
            ipc: ipc::IPCService::new().unwrap(),
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> AppConfig {
    let config = state.settings.lock().unwrap();
    config.clone()
}

#[tauri::command]
fn update_config(state: tauri::State<AppState>, new_config: AppConfig) {
    let mut config = state.settings.lock().unwrap();
    *config = new_config;
    config.write();

    state.ipc.clone().update_config().unwrap();
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Capability {
    cpu: bool,
    cuda: bool,
    vulkan: bool,
}

#[tauri::command]
fn check_capability() -> Capability {
    // cuda:
    // cudart64_12.dll
    // cublas64_12.dll

    // vulkan:
    // vulkan-1.dllの存在確認

    let mut capability = Capability {
        cpu: true,
        cuda: false,
        vulkan: false,
    };

    // Check for CUDA availability
    let cuda_files = ["cudart64_12.dll", "cublas64_12.dll"];
    let cuda_available = cuda_files.iter().all(|file| {
        // Check if the file exists in system path or in the current directory
        std::env::var("PATH")
            .unwrap_or_default()
            .split(';')
            .map(PathBuf::from)
            .chain(std::iter::once(std::env::current_dir().unwrap_or_default()))
            .any(|path| path.join(file).exists())
    });
    capability.cuda = cuda_available;

    // Check for Vulkan availability
    let vulkan_file = "vulkan-1.dll";
    let vulkan_available = std::env::var("PATH")
        .unwrap_or_default()
        .split(';')
        .map(PathBuf::from)
        .chain(std::iter::once(std::env::current_dir().unwrap_or_default()))
        .any(|path| path.join(vulkan_file).exists());
    capability.vulkan = vulkan_available;

    capability
}

#[tauri::command]
fn reset_learning(state: tauri::State<AppState>) -> Result<(), String> {
    // 学習データのディレクトリを削除
    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    let memory_dir = PathBuf::from(appdata).join("Azookey").join("memory");

    if memory_dir.exists() {
        std::fs::remove_dir_all(&memory_dir).map_err(|e| e.to_string())?;
    }

    // ディレクトリを再作成
    std::fs::create_dir_all(&memory_dir).map_err(|e| e.to_string())?;

    // IPCで変換エンジンに通知
    state
        .ipc
        .clone()
        .update_config()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ユーザー辞書関連コマンド
#[tauri::command]
fn get_user_dictionary() -> Result<UserDictionary, String> {
    Ok(UserDictionary::load())
}

#[tauri::command]
fn add_dictionary_entry(state: tauri::State<AppState>, entry: UserDictEntry) -> Result<(), String> {
    // バリデーション
    if entry.reading.trim().is_empty() {
        return Err("読みを入力してください".to_string());
    }
    if entry.word.trim().is_empty() {
        return Err("単語を入力してください".to_string());
    }

    let mut dict = UserDictionary::load();
    dict.add_entry(entry);
    dict.save()?;

    // IPCで変換エンジンに通知
    let _ = state.ipc.clone().update_config();

    Ok(())
}

#[tauri::command]
fn remove_dictionary_entry(state: tauri::State<AppState>, index: usize) -> Result<(), String> {
    let mut dict = UserDictionary::load();
    dict.remove_entry(index)?;
    dict.save()?;

    // IPCで変換エンジンに通知
    let _ = state.ipc.clone().update_config();

    Ok(())
}

#[tauri::command]
fn update_dictionary_entry(
    state: tauri::State<AppState>,
    index: usize,
    entry: UserDictEntry,
) -> Result<(), String> {
    // バリデーション
    if entry.reading.trim().is_empty() {
        return Err("読みを入力してください".to_string());
    }
    if entry.word.trim().is_empty() {
        return Err("単語を入力してください".to_string());
    }

    let mut dict = UserDictionary::load();
    dict.update_entry(index, entry)?;
    dict.save()?;

    // IPCで変換エンジンに通知
    let _ = state.ipc.clone().update_config();

    Ok(())
}

#[tauri::command]
fn export_dictionary(path: String) -> Result<(), String> {
    let dict = UserDictionary::load();

    let mut content = String::from("# ユーザー辞書エクスポート\n# 読み<TAB>単語<TAB>品詞\n");
    for entry in &dict.entries {
        content.push_str(&format!(
            "{}\t{}\t{}\n",
            entry.reading, entry.word, entry.part_of_speech
        ));
    }

    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_dictionary(
    state: tauri::State<AppState>,
    path: String,
    merge: bool,
) -> Result<usize, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let mut imported_entries = Vec::new();
    for line in content.lines() {
        // コメント行をスキップ
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            imported_entries.push(UserDictEntry {
                reading: parts[0].to_string(),
                word: parts[1].to_string(),
                part_of_speech: parts.get(2).unwrap_or(&"その他").to_string(),
            });
        }
    }

    let count = imported_entries.len();

    let mut dict = if merge {
        UserDictionary::load()
    } else {
        UserDictionary::default()
    };

    for entry in imported_entries {
        dict.add_entry(entry);
    }

    dict.save()?;

    // IPCで変換エンジンに通知
    let _ = state.ipc.clone().update_config();

    Ok(count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config,
            update_config,
            check_capability,
            reset_learning,
            get_user_dictionary,
            add_dictionary_entry,
            remove_dictionary_entry,
            update_dictionary_entry,
            export_dictionary,
            import_dictionary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
