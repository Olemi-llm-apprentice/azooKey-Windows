# カスタムキーバインド機能 設計仕様書

## 1. 概要

ユーザーが入力モード切り替えキーやその他のショートカットキーを自由にカスタマイズできる機能です。

### 1.1 背景

- 現在、モード切り替えは全角/半角キー（0xF3/0xF4）にハードコードされている
- ユーザーによってはF13/F14や他のキーを使いたい場合がある
- MagicKeyboardやカスタムキーボードでは独自のキー配置がある

### 1.2 ユースケース

1. **Mac用キーボード使用者**: 英数/かなキーの代わりにF13/F14を使用
2. **カスタムキーボード使用者**: 任意のキーにモード切り替えを割り当て
3. **効率化**: よく使う操作にショートカットを割り当て

---

## 2. 機能要件

### 2.1 カスタマイズ可能なアクション

| アクション | 説明 | デフォルトキー |
|-----------|------|---------------|
| `toggle_input_mode` | かな/英数切り替え（トグル） | 全角/半角 (0xF3/0xF4) |
| `set_kana_mode` | かなモードに切り替え | なし |
| `set_latin_mode` | 英数モードに切り替え | なし |
| `convert_hiragana` | ひらがな変換 | F6 |
| `convert_katakana` | カタカナ変換 | F7 |
| `convert_half_katakana` | 半角カタカナ変換 | F8 |
| `convert_full_latin` | 全角英数変換 | F9 |
| `convert_half_latin` | 半角英数変換 | F10 |

### 2.2 対応キー

| カテゴリ | キー | 仮想キーコード |
|---------|------|---------------|
| ファンクション | F1〜F24 | 0x70〜0x87 |
| 特殊 | 全角/半角 | 0xF3, 0xF4 |
| 特殊 | 変換 | 0x1C |
| 特殊 | 無変換 | 0x1D |
| 特殊 | かな | 0x15 |
| 修飾キー | Ctrl, Alt, Shift | 組み合わせ対応 |

### 2.3 修飾キー対応

```json
{
  "key": "F13",
  "modifiers": ["ctrl", "shift"]
}
```

---

## 3. 技術設計

### 3.1 設定スキーマ

```json
{
  "keybindings": {
    "toggle_input_mode": [
      { "key": "Zenkaku/Hankaku" },
      { "key": "F13" },
      { "key": "F14" }
    ],
    "set_kana_mode": [
      { "key": "Hiragana" }
    ],
    "set_latin_mode": [
      { "key": "Muhenkan" }
    ],
    "convert_hiragana": [
      { "key": "F6" }
    ],
    "convert_katakana": [
      { "key": "F7" }
    ],
    "convert_half_katakana": [
      { "key": "F8" }
    ],
    "convert_full_latin": [
      { "key": "F9" }
    ],
    "convert_half_latin": [
      { "key": "F10" }
    ]
  }
}
```

### 3.2 Rust設定構造体

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    #[serde(default)]
    pub toggle_input_mode: Vec<KeyBinding>,
    #[serde(default)]
    pub set_kana_mode: Vec<KeyBinding>,
    #[serde(default)]
    pub set_latin_mode: Vec<KeyBinding>,
    #[serde(default)]
    pub convert_hiragana: Vec<KeyBinding>,
    #[serde(default)]
    pub convert_katakana: Vec<KeyBinding>,
    #[serde(default)]
    pub convert_half_katakana: Vec<KeyBinding>,
    #[serde(default)]
    pub convert_full_latin: Vec<KeyBinding>,
    #[serde(default)]
    pub convert_half_latin: Vec<KeyBinding>,
}
```

### 3.3 キー名とキーコードのマッピング

```rust
fn key_name_to_code(name: &str) -> Option<u32> {
    match name.to_lowercase().as_str() {
        // ファンクションキー
        "f1" => Some(0x70),
        "f2" => Some(0x71),
        // ... F3-F12
        "f13" => Some(0x7C),
        "f14" => Some(0x7D),
        "f15" => Some(0x7E),
        // ... F16-F24
        
        // 特殊キー
        "zenkaku/hankaku" | "zenkaku" | "hankaku" => Some(0xF3),
        "henkan" | "convert" => Some(0x1C),
        "muhenkan" | "nonconvert" => Some(0x1D),
        "hiragana" | "kana" => Some(0x15),
        
        _ => None,
    }
}
```

### 3.4 IMEクライアント側の変更

`user_action.rs` の `TryFrom<usize>` を、設定から読み込んだキーバインドを参照するように変更:

```rust
impl UserAction {
    pub fn from_key_code_with_config(
        key_code: usize,
        config: &KeybindingsConfig,
    ) -> Result<UserAction> {
        // 設定からキーバインドをチェック
        if config.toggle_input_mode.iter().any(|kb| kb.matches(key_code)) {
            return Ok(UserAction::ToggleInputMode);
        }
        if config.set_kana_mode.iter().any(|kb| kb.matches(key_code)) {
            return Ok(UserAction::SetKanaMode);
        }
        if config.set_latin_mode.iter().any(|kb| kb.matches(key_code)) {
            return Ok(UserAction::SetLatinMode);
        }
        // ... 他のアクション
        
        // デフォルトの処理
        // ...
    }
}
```

---

## 4. 実装計画

### Phase 1: 基本実装 ✅ 完了

1. [x] 設計仕様書の作成
2. [x] `KeybindingsConfig` をRust共有ライブラリに追加
3. [x] `AppConfig` に `keybindings` フィールドを追加
4. [x] `user_action.rs` を設定参照型に変更
5. [x] デフォルトキーバインドの定義

### Phase 2: 設定UI ✅ 完了

1. [x] 設定アプリにキーバインド設定ページを追加
2. [x] キー入力キャプチャ機能
3. [x] キーバインドの追加/削除UI

### Phase 3: IMEクライアント統合 ✅ 完了

1. [x] `user_action.rs` に `from_key_code_with_config` メソッド追加
2. [x] `composition.rs` で設定からキーバインドを読み込み
3. [x] `SetKanaMode`/`SetLatinMode` アクション対応

### Phase 4: 拡張（将来）

1. [ ] 修飾キー対応（Ctrl, Alt, Shift）
2. [ ] キーバインドのエクスポート/インポート
3. [ ] プリセット機能

---

## 5. UI設計

### 5.1 設定画面

```
┌─────────────────────────────────────────────────┐
│ キーバインド設定                                  │
├─────────────────────────────────────────────────┤
│                                                 │
│ 入力モード切り替え:                              │
│   [全角/半角] [×]                               │
│   [F13]       [×]                               │
│   [+ キーを追加]                                 │
│                                                 │
│ かなモードに切り替え:                            │
│   [キーを設定...]                               │
│                                                 │
│ 英数モードに切り替え:                            │
│   [キーを設定...]                               │
│                                                 │
│ ─────────────────────────────────────────────── │
│ 変換キー:                                        │
│   ひらがな変換: [F6]                             │
│   カタカナ変換: [F7]                             │
│   半角ｶﾅ変換:   [F8]                             │
│   全角英数変換: [F9]                             │
│   半角英数変換: [F10]                            │
│                                                 │
│ [デフォルトに戻す]                               │
│                                                 │
└─────────────────────────────────────────────────┘
```

### 5.2 キー入力ダイアログ

```
┌─────────────────────────────────────┐
│ キーを押してください...              │
│                                     │
│         [F13]                       │
│                                     │
│ [キャンセル]            [設定する]  │
└─────────────────────────────────────┘
```

---

## 6. テスト計画

### 6.1 テスト観点表

| Case ID | Input / Precondition | Perspective | Expected Result | Notes |
|---------|---------------------|-------------|-----------------|-------|
| TC-KB-01 | F13でモード切り替え設定 | 正常系 | F13キーでモードが切り替わる | - |
| TC-KB-02 | 複数キーを同一アクションに設定 | 正常系 | どちらのキーでも動作する | - |
| TC-KB-03 | 設定なしのキー | 正常系 | デフォルト動作 | - |
| TC-KB-04 | 無効なキー名 | 異常系 | エラーまたは無視 | - |
| TC-KB-05 | 空の設定 | 境界値 | デフォルトキーバインド使用 | - |
| TC-KB-06 | 重複キー設定 | 異常系 | 最初の設定が優先 | - |

---

## 7. 仮想キーコード一覧

### ファンクションキー

| キー | コード | キー | コード |
|------|--------|------|--------|
| F1 | 0x70 | F13 | 0x7C |
| F2 | 0x71 | F14 | 0x7D |
| F3 | 0x72 | F15 | 0x7E |
| F4 | 0x73 | F16 | 0x7F |
| F5 | 0x74 | F17 | 0x80 |
| F6 | 0x75 | F18 | 0x81 |
| F7 | 0x76 | F19 | 0x82 |
| F8 | 0x77 | F20 | 0x83 |
| F9 | 0x78 | F21 | 0x84 |
| F10 | 0x79 | F22 | 0x85 |
| F11 | 0x7A | F23 | 0x86 |
| F12 | 0x7B | F24 | 0x87 |

### 日本語入力関連キー

| キー | コード | 説明 |
|------|--------|------|
| 全角/半角 | 0xF3, 0xF4 | IME切り替え |
| 変換 | 0x1C | 変換キー |
| 無変換 | 0x1D | 無変換キー |
| かな | 0x15 | かなキー |

---

このドキュメントは実装の進捗に応じて更新されます。

