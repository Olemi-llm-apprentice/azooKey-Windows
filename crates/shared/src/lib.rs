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
pub struct PredictionConfig {
    pub enabled: bool,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        PredictionConfig { enabled: true }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IikanjiConfig {
    pub enabled: bool,
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for IikanjiConfig {
    fn default() -> Self {
        IikanjiConfig {
            enabled: false,
            provider: "openai".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            max_tokens: 256,
            temperature: 0.7,
        }
    }
}

// キーバインド設定
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyBinding {
    pub key: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
}

impl KeyBinding {
    pub fn new(key: &str) -> Self {
        KeyBinding {
            key: key.to_string(),
            modifiers: Vec::new(),
        }
    }

    pub fn with_modifiers(key: &str, modifiers: Vec<&str>) -> Self {
        KeyBinding {
            key: key.to_string(),
            modifiers: modifiers.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    /// キーコードを取得
    pub fn get_key_code(&self) -> Option<u32> {
        key_name_to_code(&self.key)
    }

    /// キーコードが一致するか判定
    pub fn matches(&self, key_code: u32) -> bool {
        self.get_key_code() == Some(key_code)
    }
}

/// キー名から仮想キーコードに変換
pub fn key_name_to_code(name: &str) -> Option<u32> {
    match name.to_lowercase().as_str() {
        // ファンクションキー
        "f1" => Some(0x70),
        "f2" => Some(0x71),
        "f3" => Some(0x72),
        "f4" => Some(0x73),
        "f5" => Some(0x74),
        "f6" => Some(0x75),
        "f7" => Some(0x76),
        "f8" => Some(0x77),
        "f9" => Some(0x78),
        "f10" => Some(0x79),
        "f11" => Some(0x7A),
        "f12" => Some(0x7B),
        "f13" => Some(0x7C),
        "f14" => Some(0x7D),
        "f15" => Some(0x7E),
        "f16" => Some(0x7F),
        "f17" => Some(0x80),
        "f18" => Some(0x81),
        "f19" => Some(0x82),
        "f20" => Some(0x83),
        "f21" => Some(0x84),
        "f22" => Some(0x85),
        "f23" => Some(0x86),
        "f24" => Some(0x87),

        // 日本語入力関連キー
        "zenkaku/hankaku" | "zenkaku" | "hankaku" | "zenkakuhankaku" => Some(0xF3),
        "henkan" | "convert" => Some(0x1C),
        "muhenkan" | "nonconvert" => Some(0x1D),
        "hiragana" | "kana" | "katakana" => Some(0x15),

        _ => None,
    }
}

/// キーコードからキー名に変換
pub fn key_code_to_name(code: u32) -> Option<&'static str> {
    match code {
        0x70 => Some("F1"),
        0x71 => Some("F2"),
        0x72 => Some("F3"),
        0x73 => Some("F4"),
        0x74 => Some("F5"),
        0x75 => Some("F6"),
        0x76 => Some("F7"),
        0x77 => Some("F8"),
        0x78 => Some("F9"),
        0x79 => Some("F10"),
        0x7A => Some("F11"),
        0x7B => Some("F12"),
        0x7C => Some("F13"),
        0x7D => Some("F14"),
        0x7E => Some("F15"),
        0x7F => Some("F16"),
        0x80 => Some("F17"),
        0x81 => Some("F18"),
        0x82 => Some("F19"),
        0x83 => Some("F20"),
        0x84 => Some("F21"),
        0x85 => Some("F22"),
        0x86 => Some("F23"),
        0x87 => Some("F24"),
        0xF3 | 0xF4 => Some("Zenkaku/Hankaku"),
        0x1C => Some("Henkan"),
        0x1D => Some("Muhenkan"),
        0x15 => Some("Hiragana"),
        _ => None,
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeybindingsConfig {
    #[serde(default = "KeybindingsConfig::default_toggle_input_mode")]
    pub toggle_input_mode: Vec<KeyBinding>,
    #[serde(default)]
    pub set_kana_mode: Vec<KeyBinding>,
    #[serde(default)]
    pub set_latin_mode: Vec<KeyBinding>,
    #[serde(default = "KeybindingsConfig::default_convert_hiragana")]
    pub convert_hiragana: Vec<KeyBinding>,
    #[serde(default = "KeybindingsConfig::default_convert_katakana")]
    pub convert_katakana: Vec<KeyBinding>,
    #[serde(default = "KeybindingsConfig::default_convert_half_katakana")]
    pub convert_half_katakana: Vec<KeyBinding>,
    #[serde(default = "KeybindingsConfig::default_convert_full_latin")]
    pub convert_full_latin: Vec<KeyBinding>,
    #[serde(default = "KeybindingsConfig::default_convert_half_latin")]
    pub convert_half_latin: Vec<KeyBinding>,
}

impl KeybindingsConfig {
    fn default_toggle_input_mode() -> Vec<KeyBinding> {
        vec![KeyBinding::new("Zenkaku/Hankaku")]
    }

    fn default_convert_hiragana() -> Vec<KeyBinding> {
        vec![KeyBinding::new("F6")]
    }

    fn default_convert_katakana() -> Vec<KeyBinding> {
        vec![KeyBinding::new("F7")]
    }

    fn default_convert_half_katakana() -> Vec<KeyBinding> {
        vec![KeyBinding::new("F8")]
    }

    fn default_convert_full_latin() -> Vec<KeyBinding> {
        vec![KeyBinding::new("F9")]
    }

    fn default_convert_half_latin() -> Vec<KeyBinding> {
        vec![KeyBinding::new("F10")]
    }

    /// キーコードに対応するアクションを取得
    pub fn get_action(&self, key_code: u32) -> Option<KeyAction> {
        if self.toggle_input_mode.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::ToggleInputMode);
        }
        if self.set_kana_mode.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::SetKanaMode);
        }
        if self.set_latin_mode.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::SetLatinMode);
        }
        if self.convert_hiragana.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::ConvertHiragana);
        }
        if self.convert_katakana.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::ConvertKatakana);
        }
        if self.convert_half_katakana.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::ConvertHalfKatakana);
        }
        if self.convert_full_latin.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::ConvertFullLatin);
        }
        if self.convert_half_latin.iter().any(|kb| kb.matches(key_code)) {
            return Some(KeyAction::ConvertHalfLatin);
        }
        None
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        KeybindingsConfig {
            toggle_input_mode: Self::default_toggle_input_mode(),
            set_kana_mode: Vec::new(),
            set_latin_mode: Vec::new(),
            convert_hiragana: Self::default_convert_hiragana(),
            convert_katakana: Self::default_convert_katakana(),
            convert_half_katakana: Self::default_convert_half_katakana(),
            convert_full_latin: Self::default_convert_full_latin(),
            convert_half_latin: Self::default_convert_half_latin(),
        }
    }
}

/// キーバインドで設定可能なアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    ToggleInputMode,
    SetKanaMode,
    SetLatinMode,
    ConvertHiragana,
    ConvertKatakana,
    ConvertHalfKatakana,
    ConvertFullLatin,
    ConvertHalfLatin,
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
    pub prediction: PredictionConfig,
    pub zenzai: ZenzaiConfig,
    #[serde(default)]
    pub iikanji: IikanjiConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            version: "0.0.2".to_string(),
            learning: LearningConfig::default(),
            prediction: PredictionConfig::default(),
            zenzai: ZenzaiConfig {
                enable: false,
                profile: "".to_string(),
                backend: "cpu".to_string(),
            },
            iikanji: IikanjiConfig::default(),
            keybindings: KeybindingsConfig::default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // テスト用にカスタムパスを使用できる関数を追加
    fn load_from_path(path: &PathBuf) -> UserDictionary {
        if !path.exists() {
            return UserDictionary::default();
        }
        
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return UserDictionary::default(),
        };
        
        let mut entries = Vec::new();
        for line in content.lines() {
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

    fn save_to_path(dict: &UserDictionary, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        let mut content = String::from("# ユーザー辞書ファイル\n# 読み<TAB>単語<TAB>品詞\n");
        for entry in &dict.entries {
            content.push_str(&format!("{}\t{}\t{}\n", entry.reading, entry.word, entry.part_of_speech));
        }
        
        std::fs::write(path, content).map_err(|e| e.to_string())
    }

    // ==========================================
    // LearningConfig テスト
    // ==========================================

    #[test]
    fn tc_l_01_learning_config_default() {
        // Given: デフォルト設定を作成
        // When: LearningConfig::default() を呼び出す
        let config = LearningConfig::default();
        
        // Then: enabled は true である
        assert!(config.enabled, "デフォルトでは学習が有効であるべき");
    }

    #[test]
    fn tc_l_02_app_config_default() {
        // Given: デフォルト設定を作成
        // When: AppConfig::default() を呼び出す
        let config = AppConfig::default();
        
        // Then: 各フィールドがデフォルト値である
        assert_eq!(config.version, "0.0.2");
        assert!(config.learning.enabled);
        assert!(config.prediction.enabled);
        assert!(!config.zenzai.enable);
        assert_eq!(config.zenzai.profile, "");
        assert_eq!(config.zenzai.backend, "cpu");
        // いい感じ変換のデフォルト値
        assert!(!config.iikanji.enabled);
        assert_eq!(config.iikanji.provider, "openai");
        assert_eq!(config.iikanji.api_key, "");
        assert_eq!(config.iikanji.model, "gpt-4o-mini");
        assert_eq!(config.iikanji.max_tokens, 256);
        assert!((config.iikanji.temperature - 0.7).abs() < 0.01);
        // キーバインドのデフォルト値
        assert_eq!(config.keybindings.toggle_input_mode.len(), 1);
        assert_eq!(config.keybindings.toggle_input_mode[0].key, "Zenkaku/Hankaku");
    }

    #[test]
    fn tc_l_03_prediction_config_default() {
        // Given: デフォルト設定を作成
        // When: PredictionConfig::default() を呼び出す
        let config = PredictionConfig::default();
        
        // Then: enabled は true である
        assert!(config.enabled, "デフォルトでは予測変換が有効であるべき");
    }

    #[test]
    fn tc_p_01_prediction_config_disabled() {
        // Given: 予測変換を無効化した設定
        // When: PredictionConfig を作成して enabled を false に設定
        let config = PredictionConfig { enabled: false };
        
        // Then: enabled は false である
        assert!(!config.enabled, "予測変換が無効化されているべき");
    }

    #[test]
    fn tc_p_02_prediction_config_serialize_deserialize() {
        // Given: 予測変換設定
        let config = PredictionConfig { enabled: true };
        
        // When: JSON にシリアライズしてデシリアライズ
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PredictionConfig = serde_json::from_str(&json).unwrap();
        
        // Then: 元の値と一致する
        assert_eq!(config.enabled, deserialized.enabled);
    }

    #[test]
    fn tc_p_03_app_config_with_prediction_serialize() {
        // Given: AppConfig全体
        let config = AppConfig::default();
        
        // When: JSON にシリアライズ
        let json = serde_json::to_string(&config).unwrap();
        
        // Then: prediction フィールドが含まれる
        assert!(json.contains("\"prediction\""), "JSONにpredictionフィールドが含まれるべき");
        assert!(json.contains("\"enabled\":true"), "prediction.enabledがtrueであるべき");
    }

    // ==========================================
    // IikanjiConfig テスト
    // ==========================================

    #[test]
    fn tc_ik_01_iikanji_config_default() {
        // Given: デフォルト設定を作成
        // When: IikanjiConfig::default() を呼び出す
        let config = IikanjiConfig::default();
        
        // Then: デフォルト値が設定されている
        assert!(!config.enabled, "デフォルトではいい感じ変換は無効");
        assert_eq!(config.provider, "openai");
        assert_eq!(config.api_key, "");
        assert_eq!(config.model, "gpt-4o-mini");
        assert_eq!(config.max_tokens, 256);
        assert!((config.temperature - 0.7).abs() < 0.01);
    }

    #[test]
    fn tc_ik_02_iikanji_config_enabled() {
        // Given: いい感じ変換を有効化した設定
        // When: IikanjiConfig を作成して enabled を true に設定
        let config = IikanjiConfig {
            enabled: true,
            provider: "openai".to_string(),
            api_key: "sk-test-key".to_string(),
            model: "gpt-4o".to_string(),
            max_tokens: 512,
            temperature: 0.5,
        };
        
        // Then: 設定値が正しい
        assert!(config.enabled);
        assert_eq!(config.api_key, "sk-test-key");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.max_tokens, 512);
    }

    #[test]
    fn tc_ik_03_iikanji_config_serialize_deserialize() {
        // Given: いい感じ変換設定
        let config = IikanjiConfig {
            enabled: true,
            provider: "openai".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4o-mini".to_string(),
            max_tokens: 256,
            temperature: 0.7,
        };
        
        // When: JSON にシリアライズしてデシリアライズ
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: IikanjiConfig = serde_json::from_str(&json).unwrap();
        
        // Then: 元の値と一致する
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.provider, deserialized.provider);
        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.max_tokens, deserialized.max_tokens);
    }

    #[test]
    fn tc_ik_04_app_config_with_iikanji_serialize() {
        // Given: AppConfig全体（いい感じ変換含む）
        let mut config = AppConfig::default();
        config.iikanji.enabled = true;
        config.iikanji.api_key = "sk-test-key".to_string();
        
        // When: JSON にシリアライズ
        let json = serde_json::to_string(&config).unwrap();
        
        // Then: iikanji フィールドが含まれる
        assert!(json.contains("\"iikanji\""), "JSONにiikanjiフィールドが含まれるべき");
        assert!(json.contains("\"provider\":\"openai\""), "provider がopenaiであるべき");
    }

    #[test]
    fn tc_ik_05_iikanji_config_deserialize_with_default_api_key() {
        // Given: api_keyが省略されたJSON
        let json = r#"{"enabled":true,"provider":"openai","model":"gpt-4o-mini","max_tokens":256,"temperature":0.7}"#;
        
        // When: デシリアライズ
        let config: IikanjiConfig = serde_json::from_str(json).unwrap();
        
        // Then: api_keyはデフォルト（空文字列）
        assert_eq!(config.api_key, "");
    }

    // ==========================================
    // KeybindingsConfig テスト
    // ==========================================

    #[test]
    fn tc_kb_01_keybindings_config_default() {
        // Given: デフォルト設定を作成
        // When: KeybindingsConfig::default() を呼び出す
        let config = KeybindingsConfig::default();
        
        // Then: デフォルトのキーバインドが設定されている
        assert_eq!(config.toggle_input_mode.len(), 1);
        assert_eq!(config.toggle_input_mode[0].key, "Zenkaku/Hankaku");
        assert_eq!(config.convert_hiragana[0].key, "F6");
        assert_eq!(config.convert_katakana[0].key, "F7");
        assert_eq!(config.convert_half_katakana[0].key, "F8");
        assert_eq!(config.convert_full_latin[0].key, "F9");
        assert_eq!(config.convert_half_latin[0].key, "F10");
    }

    #[test]
    fn tc_kb_02_key_name_to_code() {
        // Given: 各種キー名
        // When: key_name_to_code を呼び出す
        // Then: 正しいキーコードが返される
        assert_eq!(key_name_to_code("F13"), Some(0x7C));
        assert_eq!(key_name_to_code("F14"), Some(0x7D));
        assert_eq!(key_name_to_code("f6"), Some(0x75));
        assert_eq!(key_name_to_code("Zenkaku/Hankaku"), Some(0xF3));
        assert_eq!(key_name_to_code("muhenkan"), Some(0x1D));
        assert_eq!(key_name_to_code("invalid_key"), None);
    }

    #[test]
    fn tc_kb_03_key_code_to_name() {
        // Given: 各種キーコード
        // When: key_code_to_name を呼び出す
        // Then: 正しいキー名が返される
        assert_eq!(key_code_to_name(0x7C), Some("F13"));
        assert_eq!(key_code_to_name(0x7D), Some("F14"));
        assert_eq!(key_code_to_name(0x75), Some("F6"));
        assert_eq!(key_code_to_name(0xF3), Some("Zenkaku/Hankaku"));
        assert_eq!(key_code_to_name(0x1D), Some("Muhenkan"));
        assert_eq!(key_code_to_name(0x00), None);
    }

    #[test]
    fn tc_kb_04_keybinding_matches() {
        // Given: F13キーバインド
        let kb = KeyBinding::new("F13");
        
        // When/Then: F13キーコード(0x7C)にマッチする
        assert!(kb.matches(0x7C));
        assert!(!kb.matches(0x7D)); // F14にはマッチしない
    }

    #[test]
    fn tc_kb_05_keybindings_get_action() {
        // Given: カスタムキーバインド設定
        let mut config = KeybindingsConfig::default();
        config.toggle_input_mode.push(KeyBinding::new("F13"));
        config.set_kana_mode.push(KeyBinding::new("F14"));
        
        // When/Then: 各キーに対応するアクションが返される
        assert_eq!(config.get_action(0x7C), Some(KeyAction::ToggleInputMode)); // F13
        assert_eq!(config.get_action(0x7D), Some(KeyAction::SetKanaMode)); // F14
        assert_eq!(config.get_action(0x75), Some(KeyAction::ConvertHiragana)); // F6
        assert_eq!(config.get_action(0x00), None); // 未設定キー
    }

    #[test]
    fn tc_kb_06_keybindings_serialize_deserialize() {
        // Given: キーバインド設定
        let mut config = KeybindingsConfig::default();
        config.toggle_input_mode.push(KeyBinding::new("F13"));
        
        // When: JSON にシリアライズしてデシリアライズ
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: KeybindingsConfig = serde_json::from_str(&json).unwrap();
        
        // Then: 元の値と一致する
        assert_eq!(config.toggle_input_mode.len(), deserialized.toggle_input_mode.len());
        assert_eq!(deserialized.toggle_input_mode[1].key, "F13");
    }

    #[test]
    fn tc_kb_07_multiple_keys_same_action() {
        // Given: 複数のキーを同じアクションに割り当て
        let mut config = KeybindingsConfig::default();
        config.toggle_input_mode.push(KeyBinding::new("F13"));
        config.toggle_input_mode.push(KeyBinding::new("F14"));
        
        // When/Then: どちらのキーでもToggleInputModeが返される
        assert_eq!(config.get_action(0xF3), Some(KeyAction::ToggleInputMode)); // Zenkaku/Hankaku
        assert_eq!(config.get_action(0x7C), Some(KeyAction::ToggleInputMode)); // F13
        assert_eq!(config.get_action(0x7D), Some(KeyAction::ToggleInputMode)); // F14
    }

    // ==========================================
    // UserDictEntry テスト
    // ==========================================

    #[test]
    fn tc_n_01_load_valid_tsv_file() {
        // Given: 有効なTSVファイルが存在
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        
        let content = "あずーきー\tazooKey\t固有名詞\nてすと\tテスト\t名詞\n";
        std::fs::write(&dict_path, content).unwrap();
        
        // When: 辞書を読み込む
        let dict = load_from_path(&dict_path);
        
        // Then: 正しくエントリが読み込まれる
        assert_eq!(dict.entries.len(), 2);
        assert_eq!(dict.entries[0].reading, "あずーきー");
        assert_eq!(dict.entries[0].word, "azooKey");
        assert_eq!(dict.entries[0].part_of_speech, "固有名詞");
    }

    #[test]
    fn tc_n_02_load_three_column_entry() {
        // Given: 3列のTSVエントリ
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        
        let content = "よみ\t単語\t品詞名\n";
        std::fs::write(&dict_path, content).unwrap();
        
        // When: 辞書を読み込む
        let dict = load_from_path(&dict_path);
        
        // Then: reading, word, part_of_speechが設定される
        assert_eq!(dict.entries.len(), 1);
        assert_eq!(dict.entries[0].reading, "よみ");
        assert_eq!(dict.entries[0].word, "単語");
        assert_eq!(dict.entries[0].part_of_speech, "品詞名");
    }

    #[test]
    fn tc_n_03_load_two_column_entry() {
        // Given: 2列のTSVエントリ
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        
        let content = "よみ\t単語\n";
        std::fs::write(&dict_path, content).unwrap();
        
        // When: 辞書を読み込む
        let dict = load_from_path(&dict_path);
        
        // Then: part_of_speechは「その他」になる
        assert_eq!(dict.entries.len(), 1);
        assert_eq!(dict.entries[0].part_of_speech, "その他");
    }

    #[test]
    fn tc_n_04_add_entry() {
        // Given: 空の辞書
        let mut dict = UserDictionary::default();
        
        // When: エントリを追加
        dict.add_entry(UserDictEntry {
            reading: "てすと".to_string(),
            word: "テスト".to_string(),
            part_of_speech: "名詞".to_string(),
        });
        
        // Then: entriesに追加される
        assert_eq!(dict.entries.len(), 1);
        assert_eq!(dict.entries[0].reading, "てすと");
    }

    #[test]
    fn tc_n_05_remove_entry() {
        // Given: 2つのエントリを持つ辞書
        let mut dict = UserDictionary::default();
        dict.add_entry(UserDictEntry {
            reading: "いち".to_string(),
            word: "一".to_string(),
            part_of_speech: "数詞".to_string(),
        });
        dict.add_entry(UserDictEntry {
            reading: "に".to_string(),
            word: "二".to_string(),
            part_of_speech: "数詞".to_string(),
        });
        
        // When: index 0を削除
        let result = dict.remove_entry(0);
        
        // Then: 指定indexのエントリが削除される
        assert!(result.is_ok());
        assert_eq!(dict.entries.len(), 1);
        assert_eq!(dict.entries[0].reading, "に");
    }

    #[test]
    fn tc_n_06_update_entry() {
        // Given: 1つのエントリを持つ辞書
        let mut dict = UserDictionary::default();
        dict.add_entry(UserDictEntry {
            reading: "ふるい".to_string(),
            word: "古い".to_string(),
            part_of_speech: "形容詞".to_string(),
        });
        
        // When: index 0を更新
        let result = dict.update_entry(0, UserDictEntry {
            reading: "あたらしい".to_string(),
            word: "新しい".to_string(),
            part_of_speech: "形容詞".to_string(),
        });
        
        // Then: 指定indexのエントリが更新される
        assert!(result.is_ok());
        assert_eq!(dict.entries[0].reading, "あたらしい");
        assert_eq!(dict.entries[0].word, "新しい");
    }

    #[test]
    fn tc_n_07_save_dictionary() {
        // Given: エントリを持つ辞書
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        
        let mut dict = UserDictionary::default();
        dict.add_entry(UserDictEntry {
            reading: "ほぞん".to_string(),
            word: "保存".to_string(),
            part_of_speech: "名詞".to_string(),
        });
        
        // When: 保存する
        let result = save_to_path(&dict, &dict_path);
        
        // Then: TSV形式で保存される
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&dict_path).unwrap();
        assert!(content.contains("ほぞん\t保存\t名詞"));
    }

    // ==========================================
    // 境界値・異常系テスト
    // ==========================================

    #[test]
    fn tc_a_01_load_nonexistent_file() {
        // Given: 辞書ファイルが存在しない
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("nonexistent.txt");
        
        // When: 辞書を読み込む
        let dict = load_from_path(&dict_path);
        
        // Then: 空のUserDictionaryを返す
        assert!(dict.entries.is_empty());
    }

    #[test]
    fn tc_a_02_load_empty_file() {
        // Given: 空のファイル
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        std::fs::write(&dict_path, "").unwrap();
        
        // When: 辞書を読み込む
        let dict = load_from_path(&dict_path);
        
        // Then: 空のUserDictionaryを返す
        assert!(dict.entries.is_empty());
    }

    #[test]
    fn tc_a_03_load_comments_only_file() {
        // Given: コメント行のみのファイル
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        let content = "# これはコメント\n# これもコメント\n\n";
        std::fs::write(&dict_path, content).unwrap();
        
        // When: 辞書を読み込む
        let dict = load_from_path(&dict_path);
        
        // Then: 空のUserDictionaryを返す
        assert!(dict.entries.is_empty());
    }

    #[test]
    fn tc_a_04_remove_invalid_index() {
        // Given: 1つのエントリを持つ辞書
        let mut dict = UserDictionary::default();
        dict.add_entry(UserDictEntry {
            reading: "てすと".to_string(),
            word: "テスト".to_string(),
            part_of_speech: "名詞".to_string(),
        });
        
        // When: 無効なindex（範囲外）で削除
        let result = dict.remove_entry(5);
        
        // Then: Err("Invalid index")
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid index");
    }

    #[test]
    fn tc_a_05_update_invalid_index() {
        // Given: 1つのエントリを持つ辞書
        let mut dict = UserDictionary::default();
        dict.add_entry(UserDictEntry {
            reading: "てすと".to_string(),
            word: "テスト".to_string(),
            part_of_speech: "名詞".to_string(),
        });
        
        // When: 無効なindex（範囲外）で更新
        let result = dict.update_entry(10, UserDictEntry {
            reading: "あたらしい".to_string(),
            word: "新しい".to_string(),
            part_of_speech: "名詞".to_string(),
        });
        
        // Then: Err("Invalid index")
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid index");
    }

    #[test]
    fn tc_a_06_remove_from_empty_dict() {
        // Given: 空のentries配列
        let mut dict = UserDictionary::default();
        
        // When: index 0で削除を試みる
        let result = dict.remove_entry(0);
        
        // Then: Err("Invalid index")
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid index");
    }

    // ==========================================
    // 追加テスト（カバレッジ向上）
    // ==========================================

    #[test]
    fn tc_n_08_load_file_with_mixed_content() {
        // Given: コメント、空行、有効なエントリが混在するファイル
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        
        let content = "# ヘッダーコメント\n\nあいう\t愛羽\t固有名詞\n# 中間コメント\nかきく\t柿食う\t動詞\n\n";
        std::fs::write(&dict_path, content).unwrap();
        
        // When: 辞書を読み込む
        let dict = load_from_path(&dict_path);
        
        // Then: 有効なエントリのみ読み込まれる
        assert_eq!(dict.entries.len(), 2);
    }

    #[test]
    fn tc_n_09_multiple_add_and_remove() {
        // Given: 空の辞書
        let mut dict = UserDictionary::default();
        
        // When: 3つ追加して1つ削除
        dict.add_entry(UserDictEntry {
            reading: "いち".to_string(),
            word: "一".to_string(),
            part_of_speech: "数詞".to_string(),
        });
        dict.add_entry(UserDictEntry {
            reading: "に".to_string(),
            word: "二".to_string(),
            part_of_speech: "数詞".to_string(),
        });
        dict.add_entry(UserDictEntry {
            reading: "さん".to_string(),
            word: "三".to_string(),
            part_of_speech: "数詞".to_string(),
        });
        let _ = dict.remove_entry(1);
        
        // Then: 2つのエントリが残り、順序が正しい
        assert_eq!(dict.entries.len(), 2);
        assert_eq!(dict.entries[0].reading, "いち");
        assert_eq!(dict.entries[1].reading, "さん");
    }

    #[test]
    fn tc_n_10_save_and_reload() {
        // Given: エントリを持つ辞書を保存
        let temp_dir = TempDir::new().unwrap();
        let dict_path = temp_dir.path().join("user_dict.txt");
        
        let mut dict = UserDictionary::default();
        dict.add_entry(UserDictEntry {
            reading: "てすと".to_string(),
            word: "テスト".to_string(),
            part_of_speech: "名詞".to_string(),
        });
        save_to_path(&dict, &dict_path).unwrap();
        
        // When: 再度読み込む
        let reloaded = load_from_path(&dict_path);
        
        // Then: 同じ内容が読み込まれる
        assert_eq!(reloaded.entries.len(), 1);
        assert_eq!(reloaded.entries[0].reading, "てすと");
        assert_eq!(reloaded.entries[0].word, "テスト");
        assert_eq!(reloaded.entries[0].part_of_speech, "名詞");
    }
}