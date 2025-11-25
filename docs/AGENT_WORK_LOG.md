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
