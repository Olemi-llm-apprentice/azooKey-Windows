# OpenAI API モデル情報

> 最終更新: 2025年11月26日（公式ドキュメントより）

## 概要

OpenAI は、ChatGPT を含む大規模言語モデルの先駆者です。API を通じて様々なモデルを提供しています。

## API エンドポイント

- **ベース URL**: `https://api.openai.com/v1`
- **Chat Completions**: `POST /chat/completions`

## 実装済みモデル（azooKey-Windows）

### GPT-5シリーズ（最新・推奨）

| モデルID | 説明 | 特徴 |
|----------|------|------|
| `gpt-5.1` | GPT-5.1 フラッグシップモデル | 最新・最高性能、エージェント・コーディング特化 |
| `gpt-5` | GPT-5 フラッグシップモデル | 高性能、安定した汎用モデル |
| `gpt-5-mini` | GPT-5 の高速・コスト効率版 | 高速、コスト効率、日常的なタスクに最適（**デフォルト**） |
| `gpt-5-nano` | GPT-5 シリーズで最も軽量 | 最速、最低コスト |

> **注意**: GPT-5シリーズは `max_tokens` ではなく `max_completion_tokens` パラメータを使用します。
> 実装では自動的に切り替わります（モデル名が `gpt-5` で始まる場合）。

### GPT-4.1（非推論モデル）

| モデルID | 説明 | 特徴 |
|----------|------|------|
| `gpt-4.1` | 最もスマートな非推論モデル | 推論機能を必要としないタスクに最適、高速 |

### GPT-4o（レガシー）

| モデルID | 説明 | 特徴 |
|----------|------|------|
| `gpt-4o` | 高速・インテリジェント・フレキシブルなGPTモデル | 依然として高性能、マルチモーダル対応 |
| `gpt-4o-mini` | 高速・低コストの小型モデル | フォーカスされたタスクに最適、コスト効率 |

## GPT-5シリーズの特徴

### 推論制御（reasoning_effort）

GPT-5シリーズでは `reasoning_effort` パラメータで推論レベルを制御可能：

| 値 | 説明 |
|----|------|
| `minimal` | 最小限の推論、最速 |
| `low` | 低レベルの推論 |
| `medium` | 中レベルの推論（デフォルト） |
| `high` | 高レベルの推論、最も精度が高い |

### no reasoning モード

GPT-5.1では推論を完全にスキップする「no reasoning」モードが利用可能で、応答速度が大幅に向上します。

## API リクエスト例

### GPT-5シリーズ

```javascript
const response = await fetch('https://api.openai.com/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${API_KEY}`
  },
  body: JSON.stringify({
    model: 'gpt-5-mini',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello!' }
    ],
    max_completion_tokens: 256,  // GPT-5では max_completion_tokens を使用
    reasoning_effort: 'medium'   // 推論レベル（オプション）
  })
});
```

### GPT-4o（レガシー）

```javascript
const response = await fetch('https://api.openai.com/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${API_KEY}`
  },
  body: JSON.stringify({
    model: 'gpt-4o',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello!' }
    ],
    max_tokens: 256,
    temperature: 0.7
  })
});
```

## ストリーミングレスポンス形式

```
data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","choices":[{"delta":{"content":"Hello"},"index":0}]}
data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","choices":[{"delta":{"content":"!"},"index":0}]}
data: [DONE]
```

## JSON モード

```javascript
{
  model: 'gpt-5-mini',
  messages: [...],
  response_format: { type: 'json_object' }
}
```

## 公式ドキュメント

- [OpenAI API リファレンス](https://platform.openai.com/docs/api-reference)
- [モデル一覧](https://platform.openai.com/docs/models)
- [GPT-5 ガイド](https://platform.openai.com/docs/guides/gpt-5)
- [料金](https://openai.com/pricing)

## 変更履歴

| 日付 | 変更内容 |
|------|---------|
| 2025-11-26 | GPT-5シリーズ（5.1, 5, 5-mini, 5-nano）、GPT-4.1を追加。デフォルトをgpt-5-miniに変更 |
