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

### 2.3 LLMプロバイダー選択

#### 対応プロバイダー

| プロバイダー | タイプ | 状態 | 特徴 |
|-------------|-------|------|------|
| **Zenzai** | ローカル | ✅ 実装済（デフォルト） | オフライン、プライバシー保護、無料 |
| **OpenAI** | クラウド | ✅ 実装済 | 高精度、APIキー必要、有料 |
| Anthropic | クラウド | 将来対応 | Claude |
| Ollama | ローカル | 将来対応 | カスタムモデル |

#### プロバイダー比較

| 項目 | Zenzai（ローカル） | OpenAI（クラウド） |
|------|-------------------|-------------------|
| インターネット | 不要 | 必要 |
| プライバシー | 完全保護 | APIに送信 |
| コスト | 無料 | API利用料金 |
| 精度 | 良好 | 最高 |
| 速度 | 高速 | ネットワーク依存 |

---

## 3. 技術設計

### 3.1 アーキテクチャ

```
                               ┌─────────────────┐
                               │   Zenzai Model  │
                               │   (ローカル)    │
                               └────────▲────────┘
                                        │
┌─────────────┐      ┌─────────────┐    │
│   IME       │─────▶│   Swift     │────┤
│   Client    │      │   Server    │    │
└─────────────┘      └─────────────┘    │
                           │            │
                           ▼            ▼
                    ┌─────────────┐  ┌─────────────┐
                    │   Settings  │  │   OpenAI    │
                    │   (Tauri)   │  │   API       │
                    └─────────────┘  └─────────────┘
```

### 3.2 設定スキーマ

```json
{
  "iikanji": {
    "enabled": true,
    "provider": "zenzai",  // "zenzai" または "openai"
    "openai": {
      "api_key": "sk-...",
      "model": "gpt-4o-mini",
      "max_tokens": 256,
      "temperature": 0.7
    }
  }
}
```

### 3.3 Rust設定構造体

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IikanjiConfig {
    pub enabled: bool,
    pub provider: String,  // "zenzai" または "openai"
    #[serde(default)]
    pub openai: OpenAIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    #[serde(default)]
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

/// OpenAIで使用可能なモデル一覧（2025年11月時点）
pub const OPENAI_MODELS: &[(&str, &str)] = &[
    // GPT-5シリーズ（最新・推奨）
    ("gpt-5.1", "GPT-5.1 (最新・最高性能)"),
    ("gpt-5", "GPT-5 (高性能・安定)"),
    ("gpt-5-mini", "GPT-5 Mini (高速・低コスト)"),  // デフォルト
    ("gpt-5-nano", "GPT-5 Nano (最速・最低コスト)"),
    // GPT-4.1（非推論モデル）
    ("gpt-4.1", "GPT-4.1 (非推論・高速)"),
    // GPT-4o系（レガシー）
    ("gpt-4o", "GPT-4o (レガシー)"),
    ("gpt-4o-mini", "GPT-4o Mini (レガシー・低コスト)"),
];
```

### 3.4 キーワード認識処理

```swift
enum IikanjiKeyword: String, CaseIterable {
    case eigo = "えいご"
    case nihongo = "にほんご"
    case emoji = "えもじ"
    case iikae = "いいかえ"
    case keigo = "けいご"
    case tamego = "ためご"
    case kousei = "こうせい"
    
    /// OpenAI用プロンプト
    var prompt: String {
        switch self {
        case .eigo:
            return "Translate the following Japanese text to English..."
        // ...
        }
    }
    
    /// Zenzai用プロンプト
    var zenzaiPrompt: String {
        switch self {
        case .eigo:
            return "を英語に翻訳:"
        // ...
        }
    }
}
```

### 3.5 プロバイダー切り替え処理

```swift
@MainActor func requestIikanji(keyword: IikanjiKeyword, context: String) -> String? {
    let provider = (config["iikanjiProvider"] as? String) ?? "zenzai"
    
    if provider == "zenzai" {
        return requestIikanjiWithZenzai(keyword: keyword, context: context)
    } else {
        return requestIikanjiWithOpenAI(keyword: keyword, context: context)
    }
}
```

### 3.6 セキュリティ考慮事項

1. **APIキーの保護**
   - APIキーは設定ファイルに保存
   - 将来的にはWindows Credential Managerを使用

2. **データプライバシー**
   - **Zenzai**: 完全ローカル処理、データ送信なし
   - **OpenAI**: 入力内容がOpenAIサーバーに送信される（警告表示）

3. **ネットワーク（OpenAI使用時）**
   - HTTPS通信のみ
   - タイムアウト設定（10秒）

---

## 4. 実装状況

### Phase 1: 基本実装 ✅ 完了

1. [x] 設計仕様書の作成
2. [x] 設定スキーマの追加 (`IikanjiConfig`, `OpenAIConfig`)
3. [x] Swift変換エンジンにLLM呼び出し機能を追加
4. [x] キーワード認識処理の実装
5. [x] プロバイダー切り替え（Zenzai/OpenAI）

### Phase 2: 設定UI ✅ 完了

1. [x] 設定アプリにいい感じ変換設定ページを追加
2. [x] プロバイダー選択UI（Zenzai/OpenAI）
3. [x] APIキー入力フォーム（OpenAI用）
4. [x] OpenAIモデル選択（最新モデル対応）
5. [x] プライバシー警告表示

### Phase 3: IMEクライアント統合 ✅ 完了

IMEで「いい感じ変換」を実際に使えるようにするための実装。

#### 3.1 gRPCサービスの追加

`crates/shared/service.proto` にいい感じ変換用のRPCを追加:

```protobuf
// いい感じ変換
rpc IsIikanjiKeyword (IsIikanjiKeywordRequest) returns (IsIikanjiKeywordResponse);
rpc IsIikanjiEnabled (IsIikanjiEnabledRequest) returns (IsIikanjiEnabledResponse);
rpc RequestIikanji (RequestIikanjiRequest) returns (RequestIikanjiResponse);
```

#### 3.2 サーバー側実装

`crates/server/src/main.rs` にFFI関数宣言とgRPCハンドラーを追加。

#### 3.3 IPCサービス

`crates/client/src/engine/ipc_service.rs` にいい感じ変換メソッドを追加:
- `is_iikanji_enabled()` - いい感じ変換が有効かを確認
- `is_iikanji_keyword(input)` - キーワード判定
- `request_iikanji(keyword, context)` - いい感じ変換実行

#### 3.4 コンテキスト管理

`crates/client/src/engine/composition.rs` で確定テキストを保持:
- `last_committed_text` フィールドを追加
- `EndComposition` 時にコンテキストを保存
- `AppendText` 時にキーワード判定といい感じ変換を実行

#### 3.5 候補ウィンドウへの統合

- いい感じ変換結果を候補リストの先頭に表示
- 結果に「🤖」マークを付けて区別

#### 3.6 実装タスク一覧

| タスクID | 内容 | 優先度 | 状態 |
|---------|------|--------|------|
| IK-IME-01 | gRPCサービス定義の追加 | 高 | [x] |
| IK-IME-02 | サーバー側FFI関数・ハンドラー追加 | 高 | [x] |
| IK-IME-03 | IPCServiceにメソッド追加 | 高 | [x] |
| IK-IME-04 | コンテキスト管理の実装 | 高 | [x] |
| IK-IME-05 | キーワード判定処理の実装 | 高 | [x] |
| IK-IME-06 | 候補ウィンドウへの結果表示 | 中 | [x] |
| IK-IME-07 | 統合テスト | 中 | [ ] |

#### 3.6 動作フロー

```
ユーザー入力: "今日は天気がいい" → [確定] → "えいご" → [変換キー]
                                    ↓
                              コンテキスト保存
                                    ↓
                            キーワード認識 ("えいご")
                                    ↓
                        RequestIikanji("えいご", "今日は天気がいい")
                                    ↓
                            Swift変換エンジン
                                    ↓
                        OpenAI API / Zenzai
                                    ↓
                    候補ウィンドウに表示: "The weather is nice today"
```

### Phase 4: 拡張（将来）

1. [ ] カスタムキーワードの追加
2. [ ] Anthropic (Claude) 対応
3. [ ] Ollama対応
4. [ ] 候補ウィンドウのUI改善（いい感じ変換専用表示）

---

## 5. テスト計画

### 5.1 ユニットテスト

| Case ID | テスト内容 | 状態 |
|---------|-----------|------|
| TC-IK-01 | IikanjiConfigデフォルト値 | ✅ |
| TC-IK-02 | IikanjiConfig有効化設定 | ✅ |
| TC-IK-03 | シリアライズ/デシリアライズ | ✅ |
| TC-IK-04 | AppConfigとの統合 | ✅ |
| TC-IK-05 | OpenAIデフォルト値 | ✅ |
| TC-IK-06 | OpenAIモデルリスト確認 | ✅ |
| TC-IK-07 | Zenzaiプロバイダー設定 | ✅ |

### 5.2 統合テスト（手動）

- Zenzaiプロバイダーでのキーワード変換
- OpenAI APIを使用したキーワード変換
- プロバイダー切り替え動作確認

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
│ プロバイダー選択:                                │
│ ┌─────────────────┐  ┌─────────────────┐       │
│ │ 🖥️ Zenzai       │  │ ☁️ OpenAI       │       │
│ │ (ローカル)      │  │ (クラウド)      │       │
│ │ ✓ オフライン   │  │ ⚠️ API必要     │       │
│ │ ✓ 無料        │  │ 高精度         │       │
│ └─────────────────┘  └─────────────────┘       │
│                                                 │
│ [OpenAI選択時のみ表示]                          │
│ APIキー: [sk-...                    ] [テスト]  │
│ モデル: [gpt-4o-mini  ▼]                        │
│                                                 │
│ ─────────────────────────────────────────────── │
│ 使用可能なキーワード:                            │
│                                                 │
│   えいご   → 英語に翻訳                          │
│   にほんご → 日本語に翻訳                        │
│   えもじ   → 絵文字を推薦                        │
│   ...                                           │
│                                                 │
│ [Zenzai選択時]                                  │
│ ✅ プライバシー保護モード                        │
│    入力内容は外部に送信されません                │
│                                                 │
│ [OpenAI選択時]                                  │
│ ⚠️ 注意: 入力内容はOpenAIに送信されます         │
│                                                 │
└─────────────────────────────────────────────────┘
```

---

## 7. 今後の検討事項

1. **コスト管理**: OpenAI API利用料金の表示・制限
2. **プライバシー**: 送信データのローカル暗号化
3. **カスタムプロンプト**: ユーザー定義のキーワードとプロンプト
4. **複数プロバイダー**: Anthropic Claude、Ollamaのサポート

---

このドキュメントは実装の進捗に応じて更新されます。

**最終更新**: 2025-11-25
