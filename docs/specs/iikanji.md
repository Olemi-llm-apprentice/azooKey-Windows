# いい感じ変換機能 設計仕様書

## 1. 概要

「いい感じ変換」は、大規模言語モデル（LLM）を活用し、特殊キーワードを入力することで、文脈に応じた翻訳、言い換え、絵文字推薦などを提供する機能です。

### 1.1 背景

- macOS版azooKeyで既に実装されている機能
- 未踏IT人材発掘・育成事業で開発された先進的な入力支援機能
- ユーザーの入力体験を大幅に向上させる

### 1.2 参考資料

- [IPA 未踏事業 成果概要](https://www.ipa.go.jp/jinzai/mitou/it/2024/nl10bi0000006qh1-att/seikashosai-ok-3.pdf)
- [azooKey-Desktop (macOS)](https://github.com/azooKey/azooKey-Desktop)

---

## 2. 機能要件

### 2.1 特殊キーワードトリガー

ユーザーが特定のキーワードを入力すると、直前の文脈をLLMに送信して変換結果を取得します。

| キーワード | 機能 | 説明 |
|-----------|------|------|
| `エイゴ` / `えいご` | 英語翻訳 | 直前の日本語文を英語に翻訳 |
| `ニホンゴ` / `にほんご` | 日本語翻訳 | 直前の英語文を日本語に翻訳 |
| `エモジ` / `えもじ` | 絵文字推薦 | 文脈に適した絵文字を推薦 |
| `イイカエ` / `いいかえ` | 言い換え | 直前の文を別の表現に言い換え |
| `ケイゴ` / `けいご` | 敬語変換 | 直前の文を敬語に変換 |
| `タメゴ` / `ためご` | タメ口変換 | 直前の文をカジュアルに変換 |
| `コウセイ` / `こうせい` | 校正 | 文法・誤字脱字の校正 |

### 2.2 動作フロー

```
1. ユーザーが「今日は天気がいい」と入力して確定
2. 続けて「エイゴ」と入力
3. システムがキーワードを認識
4. コンテキスト「今日は天気がいい」をLLMに送信
5. LLMから「The weather is nice today」を取得
6. 候補リストに変換結果を表示
7. ユーザーが候補を選択して確定
```

### 2.3 LLM連携

#### 対応LLMプロバイダー

| プロバイダー | API | 状態 |
|-------------|-----|------|
| OpenAI | Chat Completions API | 優先実装 |
| Anthropic | Messages API | 将来対応 |
| ローカルLLM | Ollama | 将来対応 |

#### API設定

```json
{
  "iikanji": {
    "enabled": true,
    "provider": "openai",
    "apiKey": "sk-...",
    "model": "gpt-4o-mini",
    "maxTokens": 256,
    "temperature": 0.7
  }
}
```

---

## 3. 技術設計

### 3.1 アーキテクチャ

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   IME       │─────▶│   Swift     │─────▶│   LLM API   │
│   Client    │      │   Server    │      │  (OpenAI)   │
└─────────────┘      └─────────────┘      └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Settings  │
                    │   (Tauri)   │
                    └─────────────┘
```

### 3.2 キーワード認識処理

Swift変換エンジンで、入力テキストがいい感じ変換キーワードかどうかを判定します。

```swift
// キーワード定義
enum IikanjiKeyword: String, CaseIterable {
    case eigo = "えいご"
    case nihongo = "にほんご"
    case emoji = "えもじ"
    case iikae = "いいかえ"
    case keigo = "けいご"
    case tamego = "ためご"
    case kousei = "こうせい"
    
    var prompt: String {
        switch self {
        case .eigo:
            return "Translate the following Japanese text to English. Output only the translation:"
        case .nihongo:
            return "以下の英語を日本語に翻訳してください。翻訳のみ出力:"
        case .emoji:
            return "以下の文脈に最も適した絵文字を1-3個提案してください。絵文字のみ出力:"
        case .iikae:
            return "以下の文を別の表現で言い換えてください。言い換えのみ出力:"
        case .keigo:
            return "以下の文を敬語に変換してください。変換結果のみ出力:"
        case .tamego:
            return "以下の文をカジュアルな口調に変換してください。変換結果のみ出力:"
        case .kousei:
            return "以下の文の文法・誤字脱字を校正してください。校正結果のみ出力:"
        }
    }
}
```

### 3.3 LLM呼び出し

非同期でLLM APIを呼び出し、結果を候補に追加します。

```swift
// OpenAI API呼び出し
func requestIikanji(keyword: IikanjiKeyword, context: String) async throws -> String {
    let url = URL(string: "https://api.openai.com/v1/chat/completions")!
    var request = URLRequest(url: url)
    request.httpMethod = "POST"
    request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    
    let body: [String: Any] = [
        "model": model,
        "messages": [
            ["role": "system", "content": keyword.prompt],
            ["role": "user", "content": context]
        ],
        "max_tokens": maxTokens,
        "temperature": temperature
    ]
    
    request.httpBody = try JSONSerialization.data(withJSONObject: body)
    
    let (data, _) = try await URLSession.shared.data(for: request)
    // レスポンスをパースして結果を返す
    ...
}
```

### 3.4 設定ファイル構造

`settings.json` に追加する設定:

```json
{
  "iikanji": {
    "enabled": true,
    "provider": "openai",
    "apiKey": "",
    "model": "gpt-4o-mini",
    "maxTokens": 256,
    "temperature": 0.7,
    "customKeywords": {}
  }
}
```

### 3.5 セキュリティ考慮事項

1. **APIキーの保護**
   - APIキーは設定ファイルに平文保存（ユーザー責任）
   - 将来的にはWindows Credential Managerを使用

2. **データプライバシー**
   - 送信されるコンテキストはLLMプロバイダーのポリシーに従う
   - ユーザーに明確な警告を表示

3. **ネットワーク**
   - HTTPS通信のみ
   - タイムアウト設定（10秒）

---

## 4. 実装計画

### Phase 1: 基本実装（優先）

1. [x] 設計仕様書の作成
2. [ ] 設定スキーマの追加 (`IikanjiConfig`)
3. [ ] Swift変換エンジンにLLM呼び出し機能を追加
4. [ ] キーワード認識処理の実装
5. [ ] 候補リストへの結果統合

### Phase 2: 設定UI

1. [ ] 設定アプリにいい感じ変換設定ページを追加
2. [ ] APIキー入力フォーム
3. [ ] プロバイダー選択
4. [ ] キーワードのカスタマイズ

### Phase 3: 拡張

1. [ ] カスタムキーワードの追加
2. [ ] 複数プロバイダー対応
3. [ ] ローカルLLM対応 (Ollama)

---

## 5. テスト計画

### 5.1 テスト観点表

| Case ID | Input / Precondition | Perspective | Expected Result | Notes |
|---------|---------------------|-------------|-----------------|-------|
| TC-IK-01 | キーワード「えいご」入力 | 正常系 | LLM呼び出しが行われる | - |
| TC-IK-02 | キーワード「エイゴ」入力（カタカナ） | 正常系 | LLM呼び出しが行われる | - |
| TC-IK-03 | コンテキストが空 | 境界値 | エラーメッセージ or 空結果 | - |
| TC-IK-04 | APIキー未設定 | 異常系 | 適切なエラー表示 | - |
| TC-IK-05 | ネットワークエラー | 異常系 | タイムアウトエラー表示 | - |
| TC-IK-06 | APIレート制限 | 異常系 | リトライ or エラー表示 | - |
| TC-IK-07 | 無効なAPIキー | 異常系 | 認証エラー表示 | - |
| TC-IK-08 | 非キーワード入力 | 正常系 | 通常の変換処理 | - |
| TC-IK-09 | enabled: false | 設定 | LLM呼び出しなし | - |
| TC-IK-10 | 長いコンテキスト（1000文字超） | 境界値 | 適切にトランケート | - |

### 5.2 統合テスト

- 実際のOpenAI APIを使用したE2Eテスト（手動）
- モックサーバーを使用した自動テスト

---

## 6. UI設計

### 6.1 設定画面

```
┌─────────────────────────────────────────────────┐
│ いい感じ変換設定                                  │
├─────────────────────────────────────────────────┤
│                                                 │
│ [✓] いい感じ変換を有効にする                     │
│                                                 │
│ LLMプロバイダー: [OpenAI        ▼]              │
│                                                 │
│ APIキー: [sk-...                    ] [テスト]  │
│                                                 │
│ モデル: [gpt-4o-mini  ▼]                        │
│                                                 │
│ ─────────────────────────────────────────────── │
│ 使用可能なキーワード:                            │
│                                                 │
│   えいご   → 英語に翻訳                          │
│   にほんご → 日本語に翻訳                        │
│   えもじ   → 絵文字を推薦                        │
│   いいかえ → 言い換え                            │
│   けいご   → 敬語に変換                          │
│   ためご   → カジュアルに変換                    │
│   こうせい → 文章を校正                          │
│                                                 │
│ ⚠️ 注意: 入力内容はLLMプロバイダーに送信されます │
│                                                 │
└─────────────────────────────────────────────────┘
```

### 6.2 候補ウィンドウ表示

いい感じ変換の結果は、通常の候補とは区別して表示:

```
┌─────────────────────────────┐
│ 🤖 The weather is nice today │  ← いい感じ変換結果
│    今日は天気が良いです      │
│    本日は好天です            │
└─────────────────────────────┘
```

---

## 7. 設定スキーマ

### 7.1 Rust (`crates/shared/src/lib.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        Self {
            enabled: false,
            provider: "openai".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            max_tokens: 256,
            temperature: 0.7,
        }
    }
}
```

---

## 8. 今後の検討事項

1. **コスト管理**: API利用料金の表示・制限
2. **プライバシー**: 送信データのローカル暗号化
3. **オフライン対応**: ローカルLLM (Ollama) のサポート
4. **カスタムプロンプト**: ユーザー定義のキーワードとプロンプト

---

このドキュメントは実装の進捗に応じて更新されます。

