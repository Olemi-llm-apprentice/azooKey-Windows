# 作業ログの記録

## 概要

現在の作業内容を `docs/AGENT_WORK_LOG.md` に記録するコマンドです。

## 前提条件

- 記録すべき作業が完了していること
- `docs/AGENT_WORK_LOG.md` が存在すること

## 実行手順

### Step 1: 現在時刻の取得

```python
python -c "from datetime import datetime; print(datetime.now().strftime('%Y-%m-%d %H:%M:%S'))"
```

### Step 2: 作業内容の整理

以下の情報を整理してください：

1. **作業内容** - 何をしたか（1行で簡潔に）
2. **実施した作業** - 具体的な作業項目（箇条書き）
3. **変更したファイル** - 変更・作成したファイル一覧
4. **備考** - 補足情報（任意）

### Step 3: ログの追記

**重要**: 必ず `search_replace` ツールを使用して追記すること。

1. `docs/AGENT_WORK_LOG.md` の末尾を読み込む
2. 最後の `---` を見つける
3. その後に新しいエントリを追加

### ログエントリのフォーマット

```markdown
---

[YYYY-MM-DD HH:MM:SS]

## 作業内容

作業の概要を1行で記述

### 実施した作業

- 作業項目1
- 作業項目2
- 作業項目3

### 変更したファイル

- `path/to/file1.ext` - 変更内容の説明
- `path/to/file2.ext` - 変更内容の説明

### 備考

補足情報があれば記述（なければ省略可）

---
```

## 注意事項

### 必須ルール

- **既存の内容を絶対に上書きしない**
- **必ず末尾に追記する**
- **PowerShellやPythonスクリプトではなく、Cursorの `search_replace` を使用**

### 記述のポイント

- **簡潔に** - 長文は避ける
- **具体的に** - 「修正した」ではなく「〇〇を△△に変更」
- **ファイルパスは正確に** - バッククォートで囲む

## 使用例

### 基本的な使用

```
ユーザー: @write-work-log

AI:
1. 現在時刻を取得
2. 直前の作業内容を整理
3. docs/AGENT_WORK_LOG.md の末尾に追記
```

### 内容を指定して使用

```
ユーザー: @write-work-log APIキー管理機能の実装を記録して

AI:
1. 現在時刻を取得
2. APIキー管理機能の実装内容を整理
3. 変更ファイル（config.js, taskpane.js等）を列挙
4. docs/AGENT_WORK_LOG.md の末尾に追記
```

### 複数の作業をまとめて記録

```
ユーザー: @write-work-log 今日の作業をまとめて記録

AI:
1. 会話履歴から今日の作業を抽出
2. 作業ごとにグループ化
3. 1つのログエントリとして追記
```

## search_replace の具体例

```typescript
// 1. ファイル末尾を確認
read_file("docs/AGENT_WORK_LOG.md", offset: lastLines)

// 2. 追記
search_replace(
  file_path: "docs/AGENT_WORK_LOG.md",
  old_string: "---\n\n",  // 最後の区切り線
  new_string: `---

[2025-11-26 01:30:00]

## 作業内容

〇〇機能の実装

### 実施した作業

- 項目1
- 項目2

### 変更したファイル

- \`src/file.js\` - 変更内容

---

`
)
```

## 参照ルール

- `.cursor/rules/work-logging.mdc` - 作業ログの詳細なガイドライン

