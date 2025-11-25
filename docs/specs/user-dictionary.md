# ユーザー辞書機能 設計仕様書

## 1. 概要

### 1.1 要望

- ユーザーが独自の単語（専門用語、人名、固有名詞など）を登録し、変換候補として利用したい
- 登録した単語を管理（追加、編集、削除）したい
- 辞書データをエクスポート/インポートして、バックアップや移行を容易にしたい

### 1.2 設計方針

AzooKeyKanaKanjiConverterの `sharedContainerURL` を活用し、ユーザー辞書データをファイルとして管理する。設定アプリ内に辞書管理画面を設け、GUI経由での単語登録・編集・削除を可能にする。

```
┌─────────────────────────────────────────────────────────────────┐
│                    ユーザー辞書システム                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   設定アプリ (Tauri)                                            │
│   ┌───────────────────────────────────────────────────────────┐ │
│   │ 辞書管理画面                                              │ │
│   │ - 単語一覧表示                                            │ │
│   │ - 新規登録 (読み, 単語, 品詞)                              │ │
│   │ - 編集                                                    │ │
│   │ - 削除                                                    │ │
│   │ - インポート/エクスポート                                  │ │
│   └───────────────────────────────────────────────────────────┘ │
│        │                                                        │
│        ▼ 辞書ファイル更新                                       │
│                                                                 │
│   %APPDATA%/Azookey/user_dictionary/                            │
│   ┌───────────────────────────────────────────────────────────┐ │
│   │ user_dict.txt (TSV形式)                                   │ │
│   │ あずーきー\tazooKey\t固有名詞                              │ │
│   │ ぎっとはぶ\tGitHub\t固有名詞                               │ │
│   │ ...                                                       │ │
│   └───────────────────────────────────────────────────────────┘ │
│        │                                                        │
│        ▼ 変換エンジン読み込み (sharedContainerURL)              │
│                                                                 │
│   変換エンジン (KanaKanjiConverter)                             │
│   ┌───────────────────────────────────────────────────────────┐ │
│   │ - システム辞書 + ユーザー辞書をマージして変換候補生成     │ │
│   │ - 登録単語を優先的に表示                                  │ │
│   └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 要件定義

### 2.1 機能要件

| 要件ID | 要件 | 優先度 |
|--------|------|--------|
| REQ-UD-001 | 単語（読み、表記、品詞）を登録できること | 必須 |
| REQ-UD-002 | 登録済み単語の一覧を表示できること | 必須 |
| REQ-UD-003 | 登録済み単語を編集できること | 必須 |
| REQ-UD-004 | 登録済み単語を削除できること | 必須 |
| REQ-UD-005 | 辞書データをエクスポートできること | 推奨 |
| REQ-UD-006 | 辞書データをインポートできること | 推奨 |
| REQ-UD-007 | 変換時に登録単語が候補として表示されること | 必須 |

### 2.2 非機能要件

| 要件ID | 要件 | 優先度 |
|--------|------|--------|
| NFR-UD-001 | 辞書データは `%APPDATA%/Azookey/user_dictionary/` に保存されること | 必須 |
| NFR-UD-002 | 辞書ファイルはTSV形式で人間が読める形式であること | 必須 |
| NFR-UD-003 | 大量の単語登録（1万件程度）でもパフォーマンスが許容範囲内であること | 推奨 |
| NFR-UD-004 | アプリケーション再起動後も辞書データが維持されること | 必須 |

---

## 3. 設計詳細

### 3.1 辞書ファイル形式

TSV（Tab-Separated Values）形式を採用：

```
# ユーザー辞書ファイル
# 読み<TAB>単語<TAB>品詞
あずーきー	azooKey	固有名詞
ぎっとはぶ	GitHub	固有名詞
みわ	Miwa	人名
せんざい	Zenzai	固有名詞
```

#### 品詞カテゴリ

| 品詞ID | 品詞名 | 説明 |
|--------|--------|------|
| 普通名詞 | 一般名詞 | 一般的な名詞 |
| 固有名詞 | 固有名詞 | 固有名詞（人名、地名、製品名など） |
| 人名 | 人名 | 人の名前 |
| 地名 | 地名 | 場所の名前 |
| 動詞 | 動詞 | 動詞 |
| 形容詞 | 形容詞 | 形容詞 |
| その他 | その他 | 上記以外 |

### 3.2 変更が必要なファイル

#### 3.2.1 server-swift/Sources/azookey-server/azookey_server.swift

**追加関数**:
```swift
// ユーザー辞書ディレクトリ
@MainActor var userDictionaryURL: URL = {
    if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
        let url = URL(filePath: appDataPath).appendingPathComponent("Azookey/user_dictionary")
        try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
    return URL(filePath: "./user_dictionary")
}()

@_silgen_name("ReloadUserDictionary")
@MainActor public func reload_user_dictionary() {
    // 変換エンジンにユーザー辞書の再読み込みを通知
    // AzooKeyKanaKanjiConverterのAPIを使用
}
```

**getOptions()の更新**:
```swift
@MainActor func getOptions(context: String = "") -> ConvertRequestOptions {
    return ConvertRequestOptions(
        // ...
        sharedContainerURL: userDictionaryURL,
        // ...
    )
}
```

#### 3.2.2 crates/shared/src/lib.rs

**追加構造体**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserDictEntry {
    pub reading: String,
    pub word: String,
    pub part_of_speech: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UserDictionary {
    pub entries: Vec<UserDictEntry>,
}

impl UserDictionary {
    pub fn load() -> Self { /* ... */ }
    pub fn save(&self) -> Result<(), String> { /* ... */ }
    pub fn add_entry(&mut self, entry: UserDictEntry) { /* ... */ }
    pub fn remove_entry(&mut self, index: usize) { /* ... */ }
    pub fn update_entry(&mut self, index: usize, entry: UserDictEntry) { /* ... */ }
}
```

#### 3.2.3 frontend/src-tauri/src/lib.rs

**追加コマンド**:
```rust
#[tauri::command]
fn get_user_dictionary() -> Result<UserDictionary, String> { /* ... */ }

#[tauri::command]
fn add_dictionary_entry(entry: UserDictEntry) -> Result<(), String> { /* ... */ }

#[tauri::command]
fn remove_dictionary_entry(index: usize) -> Result<(), String> { /* ... */ }

#[tauri::command]
fn update_dictionary_entry(index: usize, entry: UserDictEntry) -> Result<(), String> { /* ... */ }

#[tauri::command]
fn export_dictionary(path: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
fn import_dictionary(path: String) -> Result<(), String> { /* ... */ }
```

#### 3.2.4 frontend/src/pages/dictionary.tsx

設定アプリに辞書管理画面を追加：
- 単語一覧テーブル（検索、ソート機能付き）
- 新規登録ダイアログ
- 編集ダイアログ
- 削除確認ダイアログ
- インポート/エクスポートボタン

### 3.3 なぜその設計か

| 選択肢 | メリット | デメリット | 判断 |
|--------|---------|-----------|------|
| TSVファイル形式 | 人間が読める、編集しやすい、他ツールとの互換性 | 大量データでの検索が遅い可能性 | ✅ 採用 |
| SQLiteデータベース | 大量データの高速検索、複雑なクエリ | 設定が複雑、ファイル形式が不透明 | ❌ 不採用 |
| JSONファイル形式 | 構造化しやすい、プログラムから扱いやすい | 大量データで可読性が下がる | ❌ 不採用 |

**理由**: ユーザーが直接ファイルを編集する可能性もあるため、人間が読みやすいTSV形式を採用。通常の使用範囲（〜1万件程度）ではパフォーマンスも問題ない。

---

## 4. テスト仕様

### 4.1 テストケース

| テストID | シナリオ | 前提条件 | 手順 | 期待結果 |
|----------|---------|---------|------|---------|
| UD-N-001 | 単語の新規登録 | 設定アプリ起動 | 1. 辞書管理画面を開く 2. 「追加」ボタン 3. 読み、単語、品詞入力 4. 保存 | 単語が一覧に追加される |
| UD-N-002 | 登録単語の変換候補表示 | 単語登録済み | 1. IMEで登録単語の読みを入力 | 登録した単語が候補に表示される |
| UD-N-003 | 単語の編集 | 単語登録済み | 1. 一覧から単語選択 2. 「編集」ボタン 3. 変更 4. 保存 | 単語が更新される |
| UD-N-004 | 単語の削除 | 単語登録済み | 1. 一覧から単語選択 2. 「削除」ボタン 3. 確認 | 単語が一覧から削除される |
| UD-N-005 | 辞書のエクスポート | 単語登録済み | 1. 「エクスポート」ボタン 2. ファイル保存先選択 | TSVファイルが保存される |
| UD-N-006 | 辞書のインポート | TSVファイルあり | 1. 「インポート」ボタン 2. ファイル選択 | ファイルの内容が辞書に追加される |
| UD-A-001 | 不正なデータ形式のインポート | 不正なファイル | 1. 不正なファイルをインポート | エラーメッセージが表示される |
| UD-A-002 | 重複登録 | 同じ読み・単語の登録あり | 1. 同じ読み・単語で登録試行 | 警告メッセージまたは上書き確認 |
| UD-B-001 | 空の読み | - | 1. 読みを空で登録試行 | バリデーションエラー |
| UD-B-002 | 空の単語 | - | 1. 単語を空で登録試行 | バリデーションエラー |
| UD-B-003 | 特殊文字を含む単語 | - | 1. タブ、改行を含む単語を登録 | エスケープ処理または禁止 |

### 4.2 合格基準

| 基準ID | 基準 | 検証方法 |
|--------|------|---------|
| AC-UD-001 | UD-N-001〜UD-N-006がすべてパス | 手動テスト |
| AC-UD-002 | UD-A-001〜UD-B-003がすべてパス | 手動テスト |
| AC-UD-003 | 辞書ファイルが正しく保存されること | ファイル確認 |
| AC-UD-004 | 登録単語が変換候補に表示されること | 手動テスト |

---

## 5. 制限事項

| 制限 | 理由 | 回避策 |
|------|------|--------|
| 品詞の活用形は未対応 | 変換エンジンの制約 | 活用形ごとに個別登録 |
| 読みはひらがなのみ | 変換エンジンの仕様 | カタカナ読みはひらがなに変換して登録 |
| 登録直後は再起動が必要な場合あり | 辞書のホットリロード未実装 | 将来的にホットリロード対応を検討 |

---

## 6. 今後の拡張

- 辞書のクラウド同期
- カテゴリ/タグによる分類
- 複数辞書の切り替え（専門辞書、プロジェクト別辞書など）
- 辞書のホットリロード（再起動不要）
- 品詞の活用形サポート
- 他のIME辞書形式からのインポート（Google日本語入力、MS-IMEなど）

---

## 7. 実装タスク

- [ ] Rust共有ライブラリ: `UserDictionary` 構造体追加
- [ ] Tauri: 辞書管理用コマンド追加
  - [ ] `get_user_dictionary`
  - [ ] `add_dictionary_entry`
  - [ ] `remove_dictionary_entry`
  - [ ] `update_dictionary_entry`
  - [ ] `export_dictionary`
  - [ ] `import_dictionary`
- [ ] フロントエンド: 辞書管理画面実装
  - [ ] 単語一覧テーブル
  - [ ] 新規登録ダイアログ
  - [ ] 編集ダイアログ
  - [ ] 削除確認ダイアログ
  - [ ] インポート/エクスポート機能
- [ ] Swift: `sharedContainerURL` の設定更新
- [ ] サイドバーに辞書管理リンク追加
- [ ] テスト実施

---

## 更新履歴

| 日付 | 内容 |
|------|------|
| 2025-11-26 | 初版作成 |

