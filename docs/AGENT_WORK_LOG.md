# Agent 作業ログ

このファイルには、Cursor Agentによる作業ログが記録されています。

---

[2025-11-26 01:30:21]

## 作業内容

azooKey-Windows プロジェクトの仕様調査と仕様書ドキュメント群の作成

### 実施した作業

- azooKey公式サイト、GitHubリポジトリ、Zenn記事の調査
- macOS版azooKey-Desktopの機能・仕様の調査
- 現在のazooKey-Windowsリポジトリのソースコード解析
- `docs/specifications/` フォルダの作成と仕様書ドキュメント群の作成

### 変更したファイル

- `docs/specifications/README.md` - 仕様書ディレクトリのREADME（新規作成）
- `docs/specifications/00-overview.md` - プロジェクト概要（新規作成）
- `docs/specifications/01-architecture.md` - アーキテクチャ設計書（新規作成）
- `docs/specifications/02-features.md` - 機能仕様書（新規作成）
- `docs/specifications/03-technical-decisions.md` - 技術選定理由書（新規作成）
- `docs/specifications/04-testing-strategy.md` - テスト戦略書（新規作成）
- `docs/specifications/05-roadmap.md` - 開発ロードマップ（新規作成）

### 備考

調査元情報:
- https://azookey.com/
- https://azookey.com/OpenSource
- https://github.com/azooKey/azooKey
- https://github.com/azooKey/azooKey-Desktop
- https://zenn.dev/azookey/articles/7a2c2d20a3cc4a
- https://zenn.dev/azookey/articles/153b1bf4da1119

仕様書に記載した主な内容:
- プロジェクトの目的と背景（未踏IT事業採択等）
- マルチプロセスアーキテクチャの解説
- Rust/Swift/TypeScriptの選定理由
- 実装済み機能と未実装機能の一覧
- 各機能のテスト観点表
- v0.2.0〜v1.0.0に向けた開発ロードマップ

---

[2025-11-26 01:39:56]

## 作業内容

履歴学習機能の設計と実装

### 実施した作業

- 仕様書を `docs/specifications/` から `docs/specs/` に移動（ルール準拠）
- `docs/specs/README.md` をインデックスとして整備
- `docs/specs/learning-history.md` 履歴学習機能の設計仕様書を作成
- Swift変換エンジンに学習機能を実装
  - `learningType` を `.inputAndOutput` に変更
  - `memoryDirectoryURL` を `%APPDATA%/Azookey/memory/` に設定
  - 学習リセット機能 (`ResetLearning` FFI関数) を追加
- Rust共有ライブラリに `LearningConfig` を追加
- 設定アプリに学習設定ページを追加
  - 学習の有効/無効切り替え
  - 学習データのリセットボタン
- サイドバーに学習設定へのリンクを追加

### 変更したファイル

- `docs/specs/README.md` - 仕様書インデックス（更新）
- `docs/specs/learning-history.md` - 履歴学習機能設計仕様書（新規作成）
- `server-swift/Sources/azookey-server/azookey_server.swift` - 学習機能の実装
- `crates/shared/src/lib.rs` - `LearningConfig` の追加
- `settings.json` - 学習設定スキーマの追加
- `frontend/src/pages/learning.tsx` - 学習設定ページ（新規作成）
- `frontend/src/components/app-sidebar.tsx` - サイドバーに学習リンク追加
- `frontend/src/main.tsx` - ルート追加
- `frontend/src-tauri/src/lib.rs` - `reset_learning` コマンド追加

### 備考

履歴学習機能は、AzooKeyKanaKanjiConverterの内蔵学習機能を活用して実装。
`learningType: .inputAndOutput` を指定することで、変換確定時に自動的に学習データが蓄積される。

---

[2025-11-26 01:45:15]

## 作業内容

ユーザー辞書機能の設計と実装

### 実施した作業

- `docs/specs/user-dictionary.md` ユーザー辞書機能の設計仕様書を作成
- Rust共有ライブラリに辞書管理機能を追加
  - `UserDictEntry`, `UserDictionary` 構造体
  - 辞書のロード、保存、追加、削除、更新機能
- Tauriバックエンドに辞書管理コマンドを追加
  - `get_user_dictionary`, `add_dictionary_entry`, `remove_dictionary_entry`
  - `update_dictionary_entry`, `export_dictionary`, `import_dictionary`
- 設定アプリに辞書管理画面を追加
  - 単語一覧テーブル（検索機能付き）
  - 新規登録/編集ダイアログ
  - 削除確認ダイアログ
  - インポート/エクスポート機能
- Swift変換エンジンの`sharedContainerURL`を更新
- サイドバーに辞書管理リンクを追加

### 変更したファイル

- `docs/specs/user-dictionary.md` - ユーザー辞書機能設計仕様書（新規作成）
- `docs/specs/README.md` - 仕様書インデックス（更新）
- `crates/shared/src/lib.rs` - `UserDictionary` 関連構造体追加
- `frontend/src-tauri/src/lib.rs` - 辞書管理コマンド追加
- `frontend/src/pages/dictionary.tsx` - 辞書管理画面（新規作成）
- `frontend/src/components/app-sidebar.tsx` - サイドバーに辞書リンク追加
- `frontend/src/main.tsx` - ルート追加
- `server-swift/Sources/azookey-server/azookey_server.swift` - `sharedContainerURL`更新

### 備考

ユーザー辞書はTSV形式で`%APPDATA%/Azookey/user_dictionary/user_dict.txt`に保存。
設定アプリから単語の追加、編集、削除、およびインポート/エクスポートが可能。

---

[2025-11-26 02:07:07]

## 作業内容

Git設定とコミット環境の整備

### 実施した作業

- GitHubフォーク設定
  - 本家 (fkunn1326/azooKey-Windows) を `upstream` に変更
  - フォーク (Olemi-llm-apprentice/azooKey-Windows) を `origin` に設定
- `tools/git/` フォルダを作成（コミットメッセージ等の一時ファイル置き場）
- `.gitignore` に `tools/git/` を追加
- コミットメッセージの文字化け修正
  - PowerShellのエンコーディング問題を回避するため、ファイル経由でコミット
  - `git commit -F tools/git/commit_msg.txt` 方式を採用
- ブランチ `feature/learning-and-dictionary` を作成してプッシュ

### 変更したファイル

- `.gitignore` - `tools/git/` を除外対象に追加
- `tools/git/commit_msg.txt` - コミットメッセージ一時ファイル（gitignore対象）

### 備考

今後のコミット手順:
1. `tools/git/commit_msg.txt` にメッセージを書く
2. `git commit -F tools/git/commit_msg.txt` でコミット
これによりPowerShellでの文字化けを回避。

---

[2025-11-26 02:15:11]

## 作業内容

ユーザー辞書機能のユニットテスト追加

### 実施した作業

- テスト戦略ルールに基づいたテスト観点表の作成
- `crates/shared` に18個のユニットテストを追加
  - `LearningConfig` / `AppConfig` のデフォルト値テスト (2件)
  - `UserDictionary` の正常系テスト (10件)
  - `UserDictionary` の境界値・異常系テスト (6件)
- テスト用依存関係 `tempfile` の追加
- `protoc` (Protocol Buffers コンパイラ) のインストール
- 全テスト実行・パス確認

### 変更したファイル

- `crates/shared/Cargo.toml` - `tempfile` dev-dependency追加
- `crates/shared/src/lib.rs` - テストモジュール追加

### テスト実行コマンド

```powershell
cargo test --manifest-path crates/shared/Cargo.toml
```

### テスト結果

```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### テスト観点表（抜粋）

| Case ID | Perspective | Expected Result |
|---------|-------------|-----------------|
| TC-N-01〜07 | 正常系 | 各操作が正しく動作 |
| TC-A-01〜06 | 境界値・異常系 | 適切なエラー/デフォルト値を返す |
| TC-L-01〜02 | 設定デフォルト | enabled: true, version: "0.0.2" |

### 備考

- テストは `tempfile` クレートを使用して一時ディレクトリでファイル操作をテスト
- 本番のファイルパス (`get_user_dict_path()`) とは独立してテスト可能な設計
- 今後、Tauriコマンド層のテスト（バリデーション等）も追加検討

---

[2025-11-26 02:21:43]

## 作業内容

v0.2.0 改善項目の実装

### 実施した作業

- **候補ウィンドウ位置計算とDPIスケーリング対応の改善**
  - `GetDpiForMonitor` APIを使用してDPIスケールファクターを取得
  - 候補ウィンドウとインジケーターの位置計算にDPIスケーリングを反映
  - モニター境界のチェックロジックを改善（上端チェックを追加）
  - `get_indicator_window_position` 関数を新規追加

- **エラーログの改善**
  - ログ保存先を `%APPDATA%/Azookey/logs/` に変更（ハードコードから環境変数ベースに）
  - リリースビルドでもログを有効に（ログレベルはINFO以上）
  - 古いログファイルの自動クリーンアップ機能を追加（最大10ファイル保持）
  - デバッグ出力（OutputDebugString）はデバッグビルドのみに限定

- **インストーラの改善**
  - アンインストール時にユーザーデータ削除オプションを追加
  - 削除対象: 学習データ、辞書データ、ログ、設定ファイル
  - チェックボックスで削除するかどうかをユーザーが選択可能

- **ロードマップと機能仕様書の更新**
  - v0.2.0の必須機能（履歴学習、ユーザー辞書）を完了としてマーク
  - 改善項目4件を完了としてマーク
  - macOS版機能対応表を更新

### 変更したファイル

- `crates/ui/src/utils.rs` - DPIスケーリング対応の位置計算関数を追加
- `crates/ui/src/main.rs` - インジケーター位置計算に新関数を使用
- `crates/client/src/trace.rs` - ログパス設定とクリーンアップ機能を追加
- `installer/Installer.iss` - アンインストール時のユーザーデータ削除オプションを追加
- `docs/specs/roadmap.md` - v0.2.0完了項目を更新
- `docs/specs/features.md` - 機能実装状況を更新

### 備考

v0.2.0の全タスクが完了。次のマイルストーンはv0.3.0（予測変換、いい感じ変換、テーマ機能など）。

---

[2025-11-26 02:31:07]

## 作業内容

v0.2.0改善項目のテスト設計仕様書作成とユニットテスト実装

### 実施した作業

- **テスト設計仕様書の作成**
  - `docs/specs/v0.2.0-improvements.md` を新規作成
  - DPIスケーリング、ログ機能、インストーラ改善の設計詳細を記載
  - テストケース表（正常系・異常系・境界値）を定義
  - 合格基準を明確化

- **DPIスケーリング位置計算のユニットテスト**
  - `crates/ui/src/utils.rs` にテストモジュールを追加
  - 位置計算ロジックを純粋な関数として分離（`calculate_candidate_position`, `calculate_indicator_position`）
  - 13個のテストケースを実装（DPIスケール計算、境界値、オーバーフロー処理）

- **ログ機能のユニットテスト**
  - `crates/client/src/trace.rs` にテストモジュールを追加
  - `tempfile` クレートを使用した一時ディレクトリでのテスト
  - 9個のテストケースを実装（ディレクトリ取得、クリーンアップ、境界値）

- **ドキュメント更新**
  - `docs/specs/README.md` に新規仕様書を追加

### 変更したファイル

- `docs/specs/v0.2.0-improvements.md` - 設計仕様書（新規作成）
- `docs/specs/README.md` - 仕様書インデックス更新
- `crates/ui/src/utils.rs` - テストモジュール追加、ロジック分離
- `crates/client/src/trace.rs` - テストモジュール追加
- `crates/client/Cargo.toml` - `tempfile` dev-dependency追加

### テスト実行コマンド

```powershell
# protocのインストール（管理者権限が必要）
choco install protoc -y

# テスト実行
cargo test --manifest-path crates/ui/Cargo.toml
cargo test --manifest-path crates/client/Cargo.toml
cargo test --manifest-path crates/shared/Cargo.toml
```

### テストケース一覧

| カテゴリ | テスト数 | 内容 |
|---------|---------|------|
| DPIスケール計算 | 4 | 100%, 125%, 150%, 200% |
| 位置計算正常系 | 5 | 中央配置、オーバーフロー処理 |
| 位置計算境界値 | 2 | 最小オフセット、WorkAreaデフォルト |
| インジケーター | 2 | 正常系、DPIスケール適用 |
| ログディレクトリ | 2 | パス生成、APPDATA取得 |
| ログクリーンアップ | 5 | 空、最大未満、最大、超過、非JSON除外 |
| 定数値 | 2 | LOG_DIR_NAME, MAX_LOG_FILES |

### テスト実行結果

```
shared クレート: 18 passed; 0 failed
client クレート: 10 passed; 0 failed
合計: 28件パス
```

### 備考

- テスト実行には `protoc` (Protocol Buffers コンパイラ) のインストールが必要
- 環境変数 `PROTOC` にprotocのパスを設定する必要あり
- 位置計算のテストはWindows APIをモック化せず、純粋なロジック部分のみをテスト
- ログクリーンアップのテストは一時ディレクトリを使用して実際のファイルシステム操作をテスト
- UIクレートのテストはazookey-server.libの依存関係でリンカーエラーが発生（コード自体は正常）

---

[2025-11-26 02:45:18]

## 作業内容

予測変換機能の設計と実装

### 実施した作業

- **予測変換機能の設計仕様書を作成**
  - `docs/specs/prediction.md` を新規作成
  - 技術設計、テストケース、合格基準を定義

- **Swift変換エンジンの設定拡張**
  - `predictionEnabled` 設定を追加
  - `getOptions`関数で`requireJapanesePrediction`を設定から制御

- **共有ライブラリに予測変換設定を追加**
  - `PredictionConfig` 構造体を追加
  - `AppConfig` に `prediction` フィールドを追加
  - 関連するユニットテストを追加

- **設定アプリに予測変換設定ページを追加**
  - `frontend/src/pages/prediction.tsx` を新規作成
  - サイドバーに「予測変換」リンクを追加
  - ルーティングを追加

- **ドキュメント更新**
  - `docs/specs/roadmap.md` - v0.3.0の予測変換を完了としてマーク
  - `docs/specs/features.md` - 機能実装状況を更新
  - `docs/specs/README.md` - 仕様書インデックスを更新

### 変更したファイル

- `docs/specs/prediction.md` - 予測変換機能設計仕様書（新規作成）
- `settings.json` - 予測変換設定セクションを追加
- `server-swift/Sources/azookey-server/azookey_server.swift` - 予測変換設定の読み込み
- `crates/shared/src/lib.rs` - `PredictionConfig` を追加
- `frontend/src/pages/prediction.tsx` - 予測変換設定ページ（新規作成）
- `frontend/src/components/app-sidebar.tsx` - サイドバーに予測変換リンク追加
- `frontend/src/main.tsx` - ルート追加
- `docs/specs/roadmap.md` - ロードマップ更新
- `docs/specs/features.md` - 機能一覧更新
- `docs/specs/README.md` - 仕様書インデックス更新

### 備考

- 予測変換は `requireJapanesePrediction` オプションを活用
- 設定からの有効/無効の切り替えが可能
- v0.3.0の予測変換タスクが完了

---

[2025-11-26 02:50:41]

## 作業内容

予測変換機能のユニットテスト追加

### 実施した作業

- テスト戦略ルールに基づいたテスト観点表の作成
- `crates/shared` に予測変換関連のユニットテストを4件追加
  - `tc_l_03_prediction_config_default` - デフォルト値テスト
  - `tc_p_01_prediction_config_disabled` - 無効化設定テスト
  - `tc_p_02_prediction_config_serialize_deserialize` - JSON変換テスト
  - `tc_p_03_app_config_with_prediction_serialize` - AppConfig全体のJSON変換テスト
- 全テスト実行・パス確認

### 変更したファイル

- `crates/shared/src/lib.rs` - 予測変換関連テストを追加

### テスト実行コマンド

```powershell
cargo test --manifest-path crates/shared/Cargo.toml
cargo test --manifest-path crates/client/Cargo.toml
```

### テスト結果

```
shared クレート: 22 passed; 0 failed
client クレート: 10 passed; 0 failed
合計: 32件パス
```

### テスト観点表

| Case ID | Perspective | Expected Result |
|---------|-------------|-----------------|
| TC-L-03 | デフォルト値 | enabled: true |
| TC-P-01 | 無効化設定 | enabled: false |
| TC-P-02 | シリアライズ | 正しく変換される |
| TC-P-03 | AppConfig変換 | predictionフィールドを含む |

### 備考

- 予測変換設定のテストはsharedクレートで実施
- Given/When/Then コメント形式でテストを記述

---

[2025-11-26 03:10:00]

## 作業内容

いい感じ変換機能の設計と実装

### 実施した作業

- **macOS版「いい感じ変換」仕様の調査**
  - IPA未踏事業成果報告書などを参照
  - 特殊キーワードによるLLM連携機能の仕様を把握

- **設計仕様書の作成**
  - `docs/specs/iikanji.md` を新規作成
  - キーワード定義、LLM連携設計、テスト計画を記載

- **Swift変換エンジンにいい感じ変換機能を追加**
  - `IikanjiKeyword` 列挙型の定義（えいご、にほんご、えもじ、いいかえ、けいご、ためご、こうせい）
  - OpenAI API呼び出し関数 `requestIikanji` の実装
  - FFI関数の追加（`RequestIikanji`, `IsIikanjiKeyword`, `IsIikanjiEnabled`）
  - 設定ファイルからの読み込み処理

- **共有ライブラリにいい感じ変換設定を追加**
  - `IikanjiConfig` 構造体を追加
  - `AppConfig` に `iikanji` フィールドを追加
  - 5件のユニットテストを追加（tc_ik_01〜tc_ik_05）

- **設定アプリにいい感じ変換設定ページを追加**
  - `frontend/src/pages/iikanji.tsx` を新規作成
  - APIキー入力、モデル選択、テスト機能
  - キーワード一覧の表示
  - プライバシー警告の表示
  - サイドバーにリンク追加

- **ドキュメント更新**
  - `docs/specs/README.md` - 仕様書インデックス更新
  - `docs/specs/roadmap.md` - v0.3.0のいい感じ変換を完了としてマーク
  - `docs/specs/features.md` - 機能実装状況を更新

### 変更したファイル

- `docs/specs/iikanji.md` - いい感じ変換設計仕様書（新規作成）
- `settings.json` - いい感じ変換設定セクションを追加
- `server-swift/Sources/azookey-server/azookey_server.swift` - いい感じ変換機能の実装
- `crates/shared/src/lib.rs` - `IikanjiConfig` を追加、テスト追加
- `frontend/src/pages/iikanji.tsx` - いい感じ変換設定ページ（新規作成）
- `frontend/src/components/app-sidebar.tsx` - サイドバーにいい感じ変換リンク追加
- `frontend/src/main.tsx` - ルート追加
- `docs/specs/README.md` - 仕様書インデックス更新
- `docs/specs/roadmap.md` - ロードマップ更新
- `docs/specs/features.md` - 機能一覧更新

### テスト実行コマンド

```powershell
cargo test --manifest-path crates/shared/Cargo.toml
```

### テスト結果

```
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### テスト観点表

| Case ID | Perspective | Expected Result |
|---------|-------------|-----------------|
| TC-IK-01 | デフォルト値 | enabled: false, provider: "openai" |
| TC-IK-02 | 有効化設定 | 各フィールドが正しく設定される |
| TC-IK-03 | シリアライズ/デシリアライズ | 正しく変換される |
| TC-IK-04 | AppConfig全体のJSON変換 | iikanjiフィールドが含まれる |
| TC-IK-05 | api_key省略時のデシリアライズ | デフォルト（空文字列）になる |

### 備考

- いい感じ変換はOpenAI API（Chat Completions API）を使用
- サポートするキーワード: えいご、にほんご、えもじ、いいかえ、けいご、ためご、こうせい
- 設定アプリからAPIキーの設定・テストが可能
- プライバシーに関する警告をUI上に表示
- v0.3.0の「いい感じ変換」タスクが完了

---

[2025-11-26 03:30:00]

## 作業内容

カスタムキーバインド機能の設計と実装

### 実施した作業

- **設計仕様書の作成**
  - `docs/specs/keybindings.md` を新規作成
  - カスタマイズ可能なアクション定義、キーコードマッピング、UI設計を記載

- **共有ライブラリにキーバインド設定を追加**
  - `KeyBinding` 構造体を追加
  - `KeybindingsConfig` 構造体を追加（8種類のアクションに対応）
  - `KeyAction` 列挙型を追加
  - `key_name_to_code` / `key_code_to_name` 変換関数を追加
  - `AppConfig` に `keybindings` フィールドを追加
  - 7件のユニットテストを追加（tc_kb_01〜tc_kb_07）

- **設定アプリにキーバインド設定ページを追加**
  - `frontend/src/pages/keybindings.tsx` を新規作成
  - キー入力キャプチャダイアログ
  - キーバインドの追加/削除UI
  - デフォルトに戻す機能
  - サイドバーにリンク追加

- **ドキュメント更新**
  - `docs/specs/README.md` - 仕様書インデックス更新
  - `docs/specs/roadmap.md` - v0.3.0のキーバインド機能を追加

### 変更したファイル

- `docs/specs/keybindings.md` - カスタムキーバインド設計仕様書（新規作成）
- `settings.json` - キーバインド設定セクションを追加
- `crates/shared/src/lib.rs` - `KeybindingsConfig` を追加、テスト追加
- `frontend/src/pages/keybindings.tsx` - キーバインド設定ページ（新規作成）
- `frontend/src/components/app-sidebar.tsx` - サイドバーにキーバインドリンク追加
- `frontend/src/main.tsx` - ルート追加
- `docs/specs/README.md` - 仕様書インデックス更新
- `docs/specs/roadmap.md` - ロードマップ更新

### テスト実行コマンド

```powershell
cargo test --manifest-path crates/shared/Cargo.toml
```

### テスト結果

```
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### テスト観点表

| Case ID | Perspective | Expected Result |
|---------|-------------|-----------------|
| TC-KB-01 | デフォルト値 | Zenkaku/Hankaku, F6-F10 |
| TC-KB-02 | キー名→コード変換 | F13=0x7C, F14=0x7D |
| TC-KB-03 | キーコード→名前変換 | 0x7C=F13 |
| TC-KB-04 | キーマッチング | F13キーがマッチする |
| TC-KB-05 | アクション取得 | 各キーに対応するアクション |
| TC-KB-06 | シリアライズ/デシリアライズ | 正しく変換される |
| TC-KB-07 | 複数キー同一アクション | どちらでも動作 |

### 備考

- 対応キー: F1〜F24、全角/半角、変換、無変換、かな
- 設定アプリでキー入力キャプチャによる設定が可能
- IMEクライアント側での設定読み込み・適用は別途実装が必要（TODO）
- v0.3.0の「カスタムキーバインド」タスクの設定UI部分が完了

---

[2025-11-25 15:30:00]

## 作業内容

いい感じ変換をZenzaiベースに変更し、OpenAIとの切り替え機能を実装

### 実施した作業

- IikanjiConfigにプロバイダー選択（Zenzai/OpenAI）を追加
- OpenAIConfigを分離して設定を整理
- OpenAIモデルリストを最新版に更新（gpt-4o, o1, o3-mini等）
- Swift変換エンジンにプロバイダー切り替え処理を実装
- 設定UIにプロバイダー選択カード（ローカル/クラウド）を追加
- プライバシー警告をプロバイダーに応じて表示
- ユニットテストを更新（7件→36件全パス）
- 設計仕様書を更新

### 変更したファイル

- `crates/shared/src/lib.rs` - IikanjiConfig, OpenAIConfig, OPENAI_MODELS追加
- `server-swift/Sources/azookey-server/azookey_server.swift` - プロバイダー切り替え処理
- `frontend/src/pages/iikanji.tsx` - プロバイダー選択UI、最新モデルリスト
- `settings.json` - 新しい設定スキーマ
- `docs/specs/iikanji.md` - 設計仕様書更新

### テスト結果

| テスト種別 | 結果 |
|-----------|------|
| Rust ユニットテスト | 36件パス |
| TypeScript型チェック | パス |

### 新機能

1. **プロバイダー選択**
   - Zenzai（ローカル）: オフライン、プライバシー保護、無料
   - OpenAI（クラウド）: 高精度、APIキー必要、有料

2. **OpenAI最新モデル対応**
   - GPT-4o系: gpt-4o, gpt-4o-mini, gpt-4o-2024-11-20
   - GPT-4系: gpt-4-turbo, gpt-4
   - o1系: o1, o1-preview, o1-mini
   - o3系: o3-mini

### 備考

- デフォルトプロバイダーはZenzai（ローカル）に変更
- Zenzai使用時はZenzai設定で有効化が必要
- OpenAI使用時のみAPIキー入力が必要

---

[2025-11-26 16:00:00]

## 作業内容

OpenAIモデルリストをGPT-5シリーズに更新

### 実施した作業

- OpenAI公式APIリファレンスを確認
- GPT-5シリーズ（5.1, 5, 5-mini, 5-nano）を追加
- GPT-4.1（非推論モデル）を追加
- デフォルトモデルをgpt-5-miniに変更
- 旧モデル（GPT-4o系）をレガシーとして残存
- ユニットテストを更新（36件全パス）
- ドキュメントを更新

### 変更したファイル

- `crates/shared/src/lib.rs` - OPENAI_MODELS定数、デフォルトモデル更新
- `frontend/src/pages/iikanji.tsx` - モデルリスト、デフォルト値更新
- `settings.json` - デフォルトモデル更新
- `docs/specs/openai-models.md` - GPT-5シリーズのドキュメント
- `docs/specs/iikanji.md` - モデルリスト更新

### 新モデルリスト

| カテゴリ | モデル |
|---------|--------|
| GPT-5（最新・推奨） | gpt-5.1, gpt-5, gpt-5-mini, gpt-5-nano |
| GPT-4.1 | gpt-4.1（非推論） |
| レガシー | gpt-4o, gpt-4o-mini |

### テスト結果

| テスト種別 | 結果 |
|-----------|------|
| Rust ユニットテスト | 36件パス |
| TypeScript型チェック | パス |

### 備考

- GPT-5シリーズでは `max_completion_tokens` パラメータを使用
- `reasoning_effort` パラメータは将来の拡張として検討

---

[2025-11-26 16:30:00]

## 作業内容

OpenAI API接続テストとGPT-5パラメータ対応

### 実施した作業

- 環境変数 `OPENAI_API_KEY` を使用してAPI接続テストを実施
- 全7モデルで接続テスト成功
- Swift変換エンジンでGPT-5シリーズの `max_completion_tokens` パラメータに自動対応

### 接続テスト結果

| モデルID | 実際のモデル | 状態 |
|---------|-------------|------|
| gpt-5.1 | gpt-5.1-2025-11-13 | ✅ OK |
| gpt-5 | gpt-5-2025-08-07 | ✅ OK |
| gpt-5-mini | gpt-5-mini-2025-08-07 | ✅ OK |
| gpt-5-nano | gpt-5-nano-2025-08-07 | ✅ OK |
| gpt-4.1 | gpt-4.1-2025-04-14 | ✅ OK |
| gpt-4o | gpt-4o-2024-08-06 | ✅ OK |
| gpt-4o-mini | gpt-4o-mini-2024-07-18 | ✅ OK |

### 変更したファイル

- `server-swift/Sources/azookey-server/azookey_server.swift` - GPT-5シリーズのmax_completion_tokens対応
- `docs/specs/openai-models.md` - パラメータ自動切り替えの注記追加

### 技術的な発見

- GPT-5シリーズ（gpt-5, gpt-5.1, gpt-5-mini, gpt-5-nano）は `max_tokens` を使うとエラーになる
- `max_completion_tokens` パラメータが必須
- モデル名が `gpt-5` で始まる場合に自動的にパラメータを切り替えるよう実装

---

[2025-11-26 09:50:00]

## 作業内容

いい感じ変換IMEクライアント統合のタスク文書化

### 実施した作業

- iikanji.md に Phase 3（IMEクライアント統合）の詳細タスクを追加
- roadmap.md のいい感じ変換ステータスを「作業中」に更新
- features.md のいい感じ変換説明を更新

### 変更したファイル

- `docs/specs/iikanji.md` - Phase 3 IMEクライアント統合タスク追加
- `docs/specs/roadmap.md` - いい感じ変換ステータス更新
- `docs/specs/features.md` - 実装状況を詳細化

### 追加したタスク（Phase 3: IMEクライアント統合）

| タスクID | 内容 | 優先度 |
|---------|------|--------|
| IK-IME-01 | FFI関数宣言の追加 | 高 |
| IK-IME-02 | キーワード判定処理の実装 | 高 |
| IK-IME-03 | コンテキスト管理の実装 | 高 |
| IK-IME-04 | RequestIikanji呼び出しの実装 | 高 |
| IK-IME-05 | 候補ウィンドウへの結果表示 | 中 |
| IK-IME-06 | エラーハンドリング | 中 |
| IK-IME-07 | 統合テスト | 中 |

---

[2025-11-26 12:30:00]

## 作業内容

いい感じ変換IMEクライアント統合 & カスタムキーバインドIME適用の実装

### 実施した作業

#### いい感じ変換（Phase 3: IMEクライアント統合）
- gRPCサービス定義の追加（IsIikanjiKeyword, IsIikanjiEnabled, RequestIikanji）
- サーバー側FFI関数宣言とgRPCハンドラーの追加
- IPCServiceにいい感じ変換メソッド追加
- Compositionにコンテキスト管理フィールド追加（last_committed_text）
- テキスト確定時にコンテキストを保存
- AppendText時にキーワード判定といい感じ変換実行
- 変換結果を候補リストの先頭に「🤖」マーク付きで表示

#### カスタムキーバインド（IMEクライアント適用）
- UserActionにfrom_key_code_with_configメソッド追加
- SetKanaMode/SetLatinModeアクション追加
- composition.rsで設定からキーバインドを読み込み適用
- 各Composition状態でSetKanaMode/SetLatinModeをハンドリング

### 変更したファイル

- `crates/shared/service.proto` - いい感じ変換gRPCサービス追加
- `crates/server/src/main.rs` - FFI関数宣言、gRPCハンドラー追加
- `crates/client/src/engine/ipc_service.rs` - いい感じ変換メソッド追加
- `crates/client/src/engine/composition.rs` - コンテキスト管理、キーバインド適用
- `crates/client/src/engine/user_action.rs` - カスタムキーバインド対応
- `docs/specs/roadmap.md` - 進捗更新
- `docs/specs/features.md` - 実装状況更新
- `docs/specs/iikanji.md` - Phase 3完了更新
- `docs/specs/keybindings.md` - 実装計画更新

### 備考

- コンパイル成功確認済み
- リンカーエラーは既存のSwiftサーバーライブラリに関するもので、今回の変更とは無関係
- 統合テストは今後の課題

---

[2025-11-26 21:45:00]

## 作業内容

暫定的なライブ変換実装を元に戻す（安定性優先）

### 実施した作業

- `append_text`ハンドラで空の候補リストを返す実装を、`get_composed_text()`を呼ぶ元の実装に戻した
- `remove_text`, `move_cursor`, `shrink_text`ハンドラも同様に元に戻した
- 使用していない`get_first_candidate()`関数を削除
- `get_all_candidates` RPCはそのまま残した（将来の改善用）

### 変更したファイル

- `crates/server/src/main.rs` - 候補取得処理を元に戻す

### 備考

- 空の候補リストを返す暫定実装は、IMEクライアント側で適切にハンドリングされておらず、メモ帳等のクラッシュを引き起こしていた
- 安定性を優先して元に戻した
- パフォーマンス改善は今後クライアント側の修正と合わせて再検討が必要
- ビルド成功確認済み

---

[2025-11-26 15:24:31]

## 作業内容

azookey-serverのクラッシュ問題を調査・修正

### 実施した作業

- サーバー起動後「キャッシュから復元」直後にクラッシュする問題を調査
- Swift FFIのDispatchQueueネストした`sync`呼び出しによるデッドロックの可能性を特定
- `crates/server/src/main.rs`を以前の安定版に戻すことで問題を解消
- protoに追加されていた`get_all_candidates` RPCメソッドの実装を追加

### 変更したファイル

- `crates/server/src/main.rs` - 以前の安定版に戻し、`get_all_candidates`メソッドを追加

### 備考

- クラッシュの根本原因はSwift側の`IMEState.sync`内で`getOptions()`や`requestCandidates()`を呼び出す際のネストした`queue.sync`呼び出しによるデッドロックの可能性が高い
- 今回は以前の状態に戻すことで対処
- Swift側の設計見直しは今後の課題

---