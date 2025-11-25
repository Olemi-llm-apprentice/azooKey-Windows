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
        assert!(!config.zenzai.enable);
        assert_eq!(config.zenzai.profile, "");
        assert_eq!(config.zenzai.backend, "cpu");
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