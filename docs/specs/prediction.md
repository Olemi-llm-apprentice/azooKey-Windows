# 予測変換機能 設計仕様書

## 1. 概要

予測変換機能は、ユーザーの入力途中で次に入力される可能性の高い単語を予測・提示する機能です。入力効率を向上させ、よりスムーズな日本語入力体験を提供します。

## 2. 機能要件

### 2.1 基本機能

| 機能 | 説明 |
|------|------|
| 入力中予測 | 入力途中で候補を予測表示 |
| 次語予測 | 変換確定後に次の単語を予測 |
| 予測候補の選択 | Tab/矢印キーで予測候補を選択 |
| 予測の有効/無効 | 設定から予測機能を切り替え可能 |

### 2.2 予測のトリガー

1. **入力中予測**: ひらがなを入力中に、入力済み文字列に続く候補を予測
2. **次語予測**: 変換確定後、文脈に基づいて次の単語を予測

## 3. 技術設計

### 3.1 アーキテクチャ

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   IMEクライアント   │────▶│   azookey-server │────▶│ 候補ウィンドウ   │
│   (crates/client)  │     │   (server-swift) │     │  (crates/ui)    │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         │                        │
         │ SetContext             │ GetPrediction
         ▼                        ▼
    文脈情報送信              予測候補取得
```

### 3.2 Swift変換エンジン側の実装

AzooKeyKanaKanjiConverterの`requestCandidates`は、`requireJapanesePrediction: true`が設定されている場合、通常の変換候補に加えて予測候補を返します。

```swift
// 現在のオプション設定（すでに予測が有効）
ConvertRequestOptions(
    requireJapanesePrediction: true,  // 日本語予測を有効
    // ...
)
```

#### 3.2.1 次語予測の実装

変換確定後に次の単語を予測するため、新しいFFI関数を追加します：

```swift
@_silgen_name("GetPrediction")
@MainActor public func get_prediction(
    context: UnsafePointer<CChar>,
    lengthPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
```

### 3.3 gRPCプロトコル拡張

```protobuf
// azookey.proto に追加
message GetPredictionRequest {
    string context = 1;  // 直前の確定テキスト
}

message GetPredictionResponse {
    repeated string predictions = 1;  // 予測候補リスト
}

service AzookeyService {
    // 既存のメソッド...
    rpc GetPrediction(GetPredictionRequest) returns (GetPredictionResponse);
}
```

### 3.4 候補ウィンドウUI

予測候補は通常の変換候補と視覚的に区別して表示します：

```
┌──────────────────────────────┐
│ 1. 東京         [予測]       │
│ 2. 東京都       [予測]       │
│ 3. 東京駅       [予測]       │
├──────────────────────────────┤
│ 4. とうきょう   [変換]       │
│ 5. 投稿         [変換]       │
└──────────────────────────────┘
```

### 3.5 設定

```json
{
  "prediction": {
    "enabled": true,
    "showInSeparateSection": true,
    "maxPredictions": 3
  }
}
```

## 4. 実装タスク

### Phase 1: 基本的な予測表示
- [x] `requireJapanesePrediction: true` の確認（すでに設定済み）
- [ ] 予測候補を変換候補と区別して表示するUIの実装
- [ ] 設定アプリに予測変換の有効/無効を追加

### Phase 2: 次語予測
- [ ] `GetPrediction` FFI関数の実装
- [ ] gRPCプロトコルの拡張
- [ ] 変換確定後の次語予測呼び出し

### Phase 3: UX改善
- [ ] 予測候補の表示スタイル調整
- [ ] キーボードショートカットの最適化
- [ ] パフォーマンス最適化

## 5. テストケース

| Case ID | Input / Precondition | Perspective | Expected Result |
|---------|---------------------|-------------|-----------------|
| TC-N-01 | 「とうきょう」と入力 | 正常系 | 「東京」「東京都」などの予測候補が表示される |
| TC-N-02 | 「東京」を確定後 | 正常系 | 「都」「駅」などの次語予測が表示される |
| TC-N-03 | 予測候補をTabで選択 | 正常系 | 選択された予測が入力される |
| TC-A-01 | 予測機能を無効化 | 異常系 | 予測候補が表示されない |
| TC-B-01 | 空入力状態 | 境界値 | 予測候補は表示されない |
| TC-B-02 | 1文字のみ入力 | 境界値 | 予測候補が表示される（または最小文字数で制限） |

## 6. 合格基準

1. 予測候補が候補ウィンドウに表示される
2. 予測候補と変換候補が視覚的に区別できる
3. 予測機能の有効/無効が設定から切り替え可能
4. パフォーマンス: 予測候補の表示が入力に対して50ms以内

## 7. 参考情報

- [AzooKeyKanaKanjiConverter](https://github.com/azooKey/AzooKeyKanaKanjiConverter)
- [azooKey-Desktop](https://github.com/azooKey/azooKey-Desktop)
- `requireJapanesePrediction` オプションの活用

