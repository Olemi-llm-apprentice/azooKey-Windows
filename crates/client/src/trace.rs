use std::fmt::Write as _;
use std::path::PathBuf;
use tracing::field::{Field, Visit};
use tracing_core::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt};
use windows::{core::PCWSTR, Win32::System::Diagnostics::Debug::OutputDebugStringW};

use crate::extension::StringExt as _;
use crate::globals::DllModule;
use crate::tracing_chrome::{ChromeLayerBuilder, EventOrSpan};

/// ログファイルを保存するディレクトリ名
const LOG_DIR_NAME: &str = "logs";

/// 保持する最大ログファイル数
const MAX_LOG_FILES: usize = 10;

pub struct StringVisitor<'a> {
    string: &'a mut String,
}

impl<'a> Visit for StringVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            write!(self.string, "{:?}", value).unwrap();
        }
    }
}

/// ログディレクトリのパスを取得する
/// %APPDATA%/Azookey/logs/ を使用
pub(crate) fn get_log_directory() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let log_dir = PathBuf::from(appdata).join("Azookey").join(LOG_DIR_NAME);
    Some(log_dir)
}

/// 指定したベースディレクトリからログディレクトリのパスを生成する（テスト用）
#[cfg(test)]
pub(crate) fn get_log_directory_with_base(base: &std::path::Path) -> PathBuf {
    base.join("Azookey").join(LOG_DIR_NAME)
}

/// 古いログファイルを削除して、最大ファイル数を維持する
pub(crate) fn cleanup_old_logs(log_dir: &PathBuf) {
    if !log_dir.exists() {
        return;
    }

    // ログファイルを取得してソート（新しい順）
    let mut log_files: Vec<_> = match std::fs::read_dir(log_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect(),
        Err(_) => return,
    };

    // 変更日時でソート（新しい順）
    log_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    // 最大数を超えた古いファイルを削除
    for old_file in log_files.iter().skip(MAX_LOG_FILES) {
        let _ = std::fs::remove_file(old_file.path());
    }
}

pub fn setup_logger() -> anyhow::Result<()> {
    // リリースビルドでもログを有効にする（ただし、ログレベルはINFO以上）
    #[cfg(not(debug_assertions))]
    let log_level = LevelFilter::INFO;

    #[cfg(debug_assertions)]
    let log_level = LevelFilter::DEBUG;

    // ログディレクトリを取得・作成
    let log_dir = match get_log_directory() {
        Some(dir) => dir,
        None => return Ok(()), // APPDATAが取得できない場合はスキップ
    };

    // ディレクトリが存在しない場合は作成
    if let Err(_) = std::fs::create_dir_all(&log_dir) {
        return Ok(()); // ディレクトリ作成に失敗した場合はスキップ
    }

    // 古いログファイルをクリーンアップ
    cleanup_old_logs(&log_dir);

    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H.%M.%S");
    let path = log_dir.join(format!("{}.json", timestamp));

    let writer = {
        if let Ok(file) = std::fs::File::create(&path) {
            file
        } else {
            return Ok(());
        }
    };

    let builder = ChromeLayerBuilder::new()
        .file(writer)
        .include_locations(true)
        .include_args(true)
        .name_fn(Box::new(|event_or_span| match event_or_span {
            EventOrSpan::Event(event) => {
                let message = {
                    let mut message = String::new();
                    event.record(&mut StringVisitor {
                        string: &mut message,
                    });
                    message
                };

                let (level, file, line) = {
                    let metadata = event.metadata();
                    let level = metadata.level().as_str();
                    let file = metadata.file().unwrap_or_default();
                    let line = metadata.line().unwrap_or_default();

                    (level, file, line)
                };

                // デバッグビルドの場合のみOutputDebugStringに出力
                #[cfg(debug_assertions)]
                {
                    let str = format!("[{}: {}:{}] {}", level, file, line, message);
                    let wide: Vec<u16> = str.as_str().to_wide_16();
                    unsafe { OutputDebugStringW(PCWSTR(wide.as_ptr())) };
                }

                message
            }
            EventOrSpan::Span(span) => span.metadata().name().to_string(),
        }));

    let (chrome_layer, sender) = builder.build();

    DllModule::get()?.sender = Some(sender);

    // 自プロジェクトのトレースのみを記録
    let filter = Targets::new()
        .with_target("azookey_windows", log_level)
        .with_default(LevelFilter::OFF);

    tracing_subscriber::registry()
        .with(filter)
        .with(chrome_layer)
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    // ============================================
    // ログディレクトリ関連のテスト
    // ============================================

    #[test]
    fn test_get_log_directory_with_base() {
        // Given: ベースディレクトリ
        // When: get_log_directory_with_base を呼び出し
        // Then: 正しいパスが返される
        let base = std::path::Path::new("C:\\Users\\test");
        let log_dir = get_log_directory_with_base(base);

        assert!(log_dir.ends_with("Azookey\\logs") || log_dir.ends_with("Azookey/logs"));
    }

    #[test]
    fn test_get_log_directory_returns_some_when_appdata_exists() {
        // Given: APPDATA環境変数が設定されている（通常のWindows環境）
        // When: get_log_directory を呼び出し
        // Then: Some(PathBuf) が返される
        if std::env::var("APPDATA").is_ok() {
            let result = get_log_directory();
            assert!(result.is_some());
            let path = result.unwrap();
            assert!(path.to_string_lossy().contains("Azookey"));
            assert!(path.to_string_lossy().contains("logs"));
        }
    }

    // ============================================
    // ログクリーンアップのテスト
    // ============================================

    #[test]
    fn test_cleanup_old_logs_empty_directory() {
        // Given: 空のディレクトリ
        // When: cleanup_old_logs を呼び出し
        // Then: 何も起きない（エラーなし）
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        cleanup_old_logs(&log_dir);

        // エラーなく完了することを確認
        assert!(log_dir.exists());
    }

    #[test]
    fn test_cleanup_old_logs_under_max() {
        // Given: MAX_LOG_FILES未満のログファイル（5個）
        // When: cleanup_old_logs を呼び出し
        // Then: ファイルは削除されない
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // 5個のログファイルを作成
        for i in 0..5 {
            let file_path = log_dir.join(format!("test_{}.json", i));
            File::create(&file_path).unwrap();
            // ファイルの更新時刻をずらす
            thread::sleep(Duration::from_millis(10));
        }

        cleanup_old_logs(&log_dir);

        // 全ファイルが残っていることを確認
        let remaining: Vec<_> = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(remaining.len(), 5);
    }

    #[test]
    fn test_cleanup_old_logs_exactly_max() {
        // Given: ちょうどMAX_LOG_FILES個のログファイル（10個）
        // When: cleanup_old_logs を呼び出し
        // Then: ファイルは削除されない
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // 10個のログファイルを作成
        for i in 0..10 {
            let file_path = log_dir.join(format!("test_{}.json", i));
            File::create(&file_path).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        cleanup_old_logs(&log_dir);

        // 全ファイルが残っていることを確認
        let remaining: Vec<_> = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(remaining.len(), 10);
    }

    #[test]
    fn test_cleanup_old_logs_over_max() {
        // Given: MAX_LOG_FILESより多いログファイル（12個）
        // When: cleanup_old_logs を呼び出し
        // Then: 古いファイルが削除され、10個になる
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // 12個のログファイルを作成
        for i in 0..12 {
            let file_path = log_dir.join(format!("test_{:02}.json", i));
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "{{}}").unwrap(); // JSONとして有効な内容
            thread::sleep(Duration::from_millis(50)); // 更新時刻をずらす
        }

        cleanup_old_logs(&log_dir);

        // 10ファイルに削減されていることを確認
        let remaining: Vec<_> = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .collect();
        assert_eq!(remaining.len(), 10);
    }

    #[test]
    fn test_cleanup_old_logs_ignores_non_json_files() {
        // Given: JSONファイルと他の拡張子のファイル
        // When: cleanup_old_logs を呼び出し
        // Then: JSONファイルのみがカウント・削除対象
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // 11個のJSONファイルと3個の他のファイルを作成
        for i in 0..11 {
            let file_path = log_dir.join(format!("test_{:02}.json", i));
            File::create(&file_path).unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        for i in 0..3 {
            let file_path = log_dir.join(format!("test_{}.txt", i));
            File::create(&file_path).unwrap();
        }

        cleanup_old_logs(&log_dir);

        // JSONファイルは10個、txtファイルは3個残っている
        let remaining_json: Vec<_> = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .collect();
        let remaining_txt: Vec<_> = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "txt").unwrap_or(false))
            .collect();

        assert_eq!(remaining_json.len(), 10);
        assert_eq!(remaining_txt.len(), 3);
    }

    #[test]
    fn test_cleanup_old_logs_nonexistent_directory() {
        // Given: 存在しないディレクトリ
        // When: cleanup_old_logs を呼び出し
        // Then: エラーなく終了
        let nonexistent = PathBuf::from("C:\\nonexistent\\path\\that\\does\\not\\exist");

        cleanup_old_logs(&nonexistent);
        // パニックしないことを確認
    }

    // ============================================
    // 定数値のテスト
    // ============================================

    #[test]
    fn test_log_dir_name_constant() {
        assert_eq!(LOG_DIR_NAME, "logs");
    }

    #[test]
    fn test_max_log_files_constant() {
        assert_eq!(MAX_LOG_FILES, 10);
    }
}
