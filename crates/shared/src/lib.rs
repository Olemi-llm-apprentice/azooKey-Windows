use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/azookey.rs"));
    include!(concat!(env!("OUT_DIR"), "/window.rs"));
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("azookey_service_descriptor");
}

fn get_config_root() -> PathBuf {
    let appdata = PathBuf::from(std::env::var("APPDATA").unwrap());
    appdata.join("Azookey")
}

const SETTINGS_FILENAME: &str = "settings.json";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LearningConfig {
    pub enabled: bool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        LearningConfig { enabled: true }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ZenzaiConfig {
    pub enable: bool,
    pub profile: String,
    pub backend: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub version: String,
    pub learning: LearningConfig,
    pub zenzai: ZenzaiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            version: "0.0.2".to_string(),
            learning: LearningConfig::default(),
            zenzai: ZenzaiConfig {
                enable: false,
                profile: "".to_string(),
                backend: "cpu".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub fn write(&self) {
        let config_path = get_config_root().join(SETTINGS_FILENAME);
        let config_str = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(config_path, config_str).unwrap();
    }

    pub fn read() -> Self {
        let config_path = get_config_root().join(SETTINGS_FILENAME);
        if !config_path.exists() {
            return AppConfig::default();
        }
        let config_str = std::fs::read_to_string(config_path).unwrap();
        serde_json::from_str(&config_str).unwrap()
    }

    pub fn new() -> Self {
        let config_path = get_config_root();
        if !config_path.exists() {
            std::fs::create_dir_all(&config_path).unwrap();
        }
        let config = AppConfig::read();
        config.write();
        config
    }
}

// ユーザー辞書エントリ
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserDictEntry {
    pub reading: String,
    pub word: String,
    pub part_of_speech: String,
}

// ユーザー辞書
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UserDictionary {
    pub entries: Vec<UserDictEntry>,
}

const USER_DICT_FILENAME: &str = "user_dict.txt";

fn get_user_dict_path() -> PathBuf {
    get_config_root().join("user_dictionary").join(USER_DICT_FILENAME)
}

impl UserDictionary {
    pub fn load() -> Self {
        let dict_path = get_user_dict_path();
        if !dict_path.exists() {
            return UserDictionary::default();
        }
        
        let content = match std::fs::read_to_string(&dict_path) {
            Ok(c) => c,
            Err(_) => return UserDictionary::default(),
        };
        
        let mut entries = Vec::new();
        for line in content.lines() {
            // コメント行をスキップ
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                entries.push(UserDictEntry {
                    reading: parts[0].to_string(),
                    word: parts[1].to_string(),
                    part_of_speech: parts.get(2).unwrap_or(&"その他").to_string(),
                });
            }
        }
        
        UserDictionary { entries }
    }
    
    pub fn save(&self) -> Result<(), String> {
        let dict_path = get_user_dict_path();
        
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = dict_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        let mut content = String::from("# ユーザー辞書ファイル\n# 読み<TAB>単語<TAB>品詞\n");
        for entry in &self.entries {
            content.push_str(&format!("{}\t{}\t{}\n", entry.reading, entry.word, entry.part_of_speech));
        }
        
        std::fs::write(&dict_path, content).map_err(|e| e.to_string())
    }
    
    pub fn add_entry(&mut self, entry: UserDictEntry) {
        self.entries.push(entry);
    }
    
    pub fn remove_entry(&mut self, index: usize) -> Result<(), String> {
        if index >= self.entries.len() {
            return Err("Invalid index".to_string());
        }
        self.entries.remove(index);
        Ok(())
    }
    
    pub fn update_entry(&mut self, index: usize, entry: UserDictEntry) -> Result<(), String> {
        if index >= self.entries.len() {
            return Err("Invalid index".to_string());
        }
        self.entries[index] = entry;
        Ok(())
    }
}
