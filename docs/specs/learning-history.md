# 履歴学習機能 設計仕様書

## 1. 概要

### 1.1 要望

- ユーザーが選択・確定した変換結果を学習し、次回以降の変換で優先的に表示したい
- 同じ読みに対して、過去に選択した候補を優先的に提示することで入力効率を向上させたい
- 必要に応じて学習データをリセットできるようにしたい

### 1.2 設計方針

AzooKeyKanaKanjiConverterが提供する学習機能（`learningType`パラメータ）を活用し、変換確定時に自動的に学習データを蓄積する。学習データは `%APPDATA%/Azookey/memory/` に保存され、永続化される。

```
┌─────────────────────────────────────────────────────────────────┐
│                        変換フロー                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   入力 "nihongo"                                                │
│        │                                                        │
│        ▼                                                        │
│   変換エンジン (KanaKanjiConverter)                             │
│   ┌───────────────────────────────────────────────────────────┐ │
│   │ 1. 辞書検索                                               │ │
│   │ 2. 学習データ参照 ← memoryDirectoryURL                    │ │
│   │ 3. 候補ランキング（学習済み候補を優先）                    │ │
│   └───────────────────────────────────────────────────────────┘ │
│        │                                                        │
│        ▼                                                        │
│   候補表示: [日本語, 二本語, にほんご, ...]                     │
│   ※「日本語」が過去に選択されていれば上位に                     │
│        │                                                        │
│        ▼ ユーザーが「日本語」を選択して確定                     │
│                                                                 │
│   学習データ更新 (learningType: .inputAndOutput)                │
│   ┌───────────────────────────────────────────────────────────┐ │
│   │ 「にほんご」→「日本語」の対応を記録                       │ │
│   │ → memoryDirectoryURL に保存                               │ │
│   └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 要件定義

### 2.1 機能要件

| 要件ID | 要件 | 優先度 |
|--------|------|--------|
| REQ-LH-001 | 変換確定時に入力と出力のペアを学習データとして保存できること | 必須 |
| REQ-LH-002 | 次回以降の変換で学習済み候補を優先的に表示できること | 必須 |
| REQ-LH-003 | 学習データをリセット（全削除）できること | 必須 |
| REQ-LH-004 | 学習データの上限を設定できること | 推奨 |
| REQ-LH-005 | 学習機能の有効/無効を切り替えられること | 任意 |

### 2.2 非機能要件

| 要件ID | 要件 | 優先度 |
|--------|------|--------|
| NFR-LH-001 | 学習データの保存は変換のパフォーマンスに影響を与えないこと（非同期保存） | 必須 |
| NFR-LH-002 | 学習データは `%APPDATA%/Azookey/memory/` に保存されること | 必須 |
| NFR-LH-003 | アプリケーション再起動後も学習データが維持されること | 必須 |
| NFR-LH-004 | 学習データの上限は65536件とすること（デフォルト） | 推奨 |

---

## 3. 設計詳細

### 3.1 AzooKeyKanaKanjiConverter の学習機能

AzooKeyKanaKanjiConverterの `ConvertRequestOptions` には以下の学習関連パラメータがある：

```swift
ConvertRequestOptions(
    // 学習タイプ
    learningType: .inputAndOutput,  // 入力と出力の両方を学習
    
    // 学習データの最大件数
    maxMemoryCount: 65536,
    
    // 学習データのリセットフラグ
    shouldResetMemory: false,
    
    // 学習データの保存ディレクトリ
    memoryDirectoryURL: URL(filePath: "..."),
    
    // ...
)
```

#### learningType の選択肢

| 値 | 説明 |
|----|------|
| `.nothing` | 学習しない（現在の設定） |
| `.inputAndOutput` | 入力と出力を両方学習 |
| `.onlyOutput` | 出力のみ学習 |

### 3.2 変更が必要なファイル

#### 3.2.1 server-swift/Sources/azookey-server/azookey_server.swift

**変更前**:
```swift
@MainActor func getOptions(context: String = "") -> ConvertRequestOptions {
    return ConvertRequestOptions(
        // ...
        learningType: .nothing,
        memoryDirectoryURL: URL(filePath: "./test"),
        // ...
    )
}
```

**変更後**:
```swift
@MainActor var memoryURL: URL = {
    if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
        return URL(filePath: appDataPath).appendingPathComponent("Azookey/memory")
    }
    return URL(filePath: "./memory")
}()

@MainActor func getOptions(context: String = "") -> ConvertRequestOptions {
    let learningEnabled = (config["learningEnabled"] as? Bool) ?? true
    let shouldReset = (config["shouldResetMemory"] as? Bool) ?? false
    
    return ConvertRequestOptions(
        // ...
        learningType: learningEnabled ? .inputAndOutput : .nothing,
        maxMemoryCount: 65536,
        shouldResetMemory: shouldReset,
        memoryDirectoryURL: memoryURL,
        sharedContainerURL: memoryURL,
        // ...
    )
}
```

#### 3.2.2 settings.json 拡張

```json
{
    "version": "0.0.2",
    "learning": {
        "enabled": true,
        "maxMemoryCount": 65536
    },
    "zenzai": {
        "enable": false,
        "profile": "",
        "backend": "cpu"
    }
}
```

#### 3.2.3 設定アプリ (frontend)

新しい設定項目を追加:
- 学習機能の有効/無効トグル
- 学習データのリセットボタン

### 3.3 なぜその設計か

| 選択肢 | メリット | デメリット | 判断 |
|--------|---------|-----------|------|
| KanaKanjiConverter内蔵の学習機能を使用 | 実装が簡単、変換ロジックとの統合が自然 | カスタマイズ性が限定的 | ✅ 採用 |
| 独自に学習データベースを実装 | 柔軟なカスタマイズが可能 | 実装コストが高い、バグのリスク | ❌ 不採用 |

**理由**: AzooKeyKanaKanjiConverterは既に学習機能を内蔵しており、パラメータを変更するだけで利用できる。独自実装は工数が大きく、変換ロジックとの統合も複雑になるため、既存機能の活用を選択。

---

## 4. テスト仕様

### 4.1 テストケース

| テストID | シナリオ | 前提条件 | 手順 | 期待結果 |
|----------|---------|---------|------|---------|
| LH-N-001 | 基本的な学習動作 | 学習機能有効、学習データなし | 1. "nihongo"入力 2. 「日本語」を選択確定 3. 再度"nihongo"入力 | 「日本語」が第1候補に表示 |
| LH-N-002 | 学習データの永続化 | 学習データあり | 1. IMEを再起動 2. "nihongo"入力 | 学習済み候補が優先表示される |
| LH-N-003 | 学習データのリセット | 学習データあり | 1. 設定でリセット実行 2. "nihongo"入力 | デフォルトの候補順序に戻る |
| LH-A-001 | 学習機能無効時 | 学習機能無効 | 1. "nihongo"入力 2. 候補選択確定 3. 再度入力 | 学習されない（候補順序変わらず） |
| LH-A-002 | memoryディレクトリ不存在 | ディレクトリ未作成 | 1. 変換実行 | 自動でディレクトリ作成され正常動作 |
| LH-B-001 | 学習データ上限到達 | 65536件の学習データ | 1. 新規変換確定 | 古いデータが削除され新規が追加 |

### 4.2 合格基準

| 基準ID | 基準 | 検証方法 |
|--------|------|---------|
| AC-LH-001 | LH-N-001〜LH-N-003がすべてパス | 手動テスト |
| AC-LH-002 | LH-A-001〜LH-B-001がすべてパス | 手動テスト |
| AC-LH-003 | 学習データリセット後、ディレクトリが空になること | ファイル確認 |
| AC-LH-004 | `%APPDATA%/Azookey/memory/` に学習ファイルが生成されること | ファイル確認 |

---

## 5. 制限事項

| 制限 | 理由 | 回避策 |
|------|------|--------|
| 学習データは端末ごとに独立 | クラウド同期未対応 | 将来的に同期機能を検討 |
| 学習データのエクスポート/インポート不可 | 現時点では優先度低 | 将来バージョンで検討 |
| 個別の学習データ削除不可 | KanaKanjiConverterの制約 | 全リセットのみ対応 |

---

## 6. 今後の拡張

- 学習データのエクスポート/インポート機能
- クラウド同期（複数端末での学習データ共有）
- 個別の学習データ削除機能
- 学習の強度調整（どの程度優先するか）

---

## 7. 実装タスク

- [ ] `server-swift/Sources/azookey-server/azookey_server.swift` の修正
  - [ ] `memoryDirectoryURL` を `%APPDATA%/Azookey/memory/` に設定
  - [ ] `learningType` を `.inputAndOutput` に変更
  - [ ] 設定からの `shouldResetMemory` 読み込み
- [ ] `settings.json` スキーマ拡張
  - [ ] `learning.enabled` 追加
  - [ ] `learning.maxMemoryCount` 追加
- [ ] 設定アプリ (frontend) 更新
  - [ ] 学習設定ページ追加
  - [ ] リセットボタン実装
- [ ] gRPC通信拡張
  - [ ] `ResetLearning` RPC追加（必要に応じて）
- [ ] テスト実施

---

## 更新履歴

| 日付 | 内容 |
|------|------|
| 2025-11-26 | 初版作成 |

