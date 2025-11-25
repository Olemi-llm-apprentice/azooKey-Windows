# azooKey-Windows テスト戦略書

## 1. 概要

本ドキュメントでは、azooKey-Windowsの品質保証のためのテスト戦略を定義します。

---

## 2. テストレベル

### 2.1 単体テスト (Unit Test)

**対象**: 個別の関数・モジュール

| コンポーネント | テスト対象 | ツール |
|---------------|-----------|--------|
| server-swift | 変換ロジック | Swift XCTest |
| crates/* | Rustモジュール | cargo test |
| frontend | Reactコンポーネント | Vitest/Jest |

### 2.2 統合テスト (Integration Test)

**対象**: コンポーネント間の連携

| 連携 | テスト内容 |
|------|-----------|
| gRPC Server ↔ Swift Engine | FFI呼び出しの正常性 |
| IME Client ↔ gRPC Server | 変換リクエスト/レスポンス |
| UI Process ↔ gRPC Server | 候補ウィンドウ更新 |

### 2.3 システムテスト (System Test)

**対象**: エンドツーエンドの動作

| シナリオ | 確認内容 |
|----------|---------|
| 入力→変換→確定 | 基本フローの正常動作 |
| アプリ切り替え | IME状態の維持 |
| システム起動 | 自動起動と初期化 |

### 2.4 互換性テスト (Compatibility Test)

**対象**: 各種アプリケーションとの互換性

| アプリケーション | テスト観点 |
|-----------------|-----------|
| メモ帳 | 基本入力 |
| Microsoft Word | リッチテキスト |
| Chrome/Edge | Webフォーム |
| Visual Studio Code | コードエディタ |
| ゲーム（チャット） | 特殊な入力コンテキスト |

---

## 3. 機能別テスト観点表

### 3.1 ライブ変換

| Case ID | Input / Precondition | Perspective | Expected Result | Notes |
|---------|---------------------|-------------|-----------------|-------|
| LC-N-01 | "a"入力 | 正常系 | "あ"に変換され表示 | 基本動作 |
| LC-N-02 | "nihongo"入力 | 正常系 | "日本語"候補が表示 | 単語変換 |
| LC-N-03 | 高速連続入力 | 正常系 | 入力に追従して変換 | パフォーマンス |
| LC-A-01 | 空入力でSpace | 異常系 | 何も起きない | 空状態 |
| LC-A-02 | 変換中にアプリ切替 | 異常系 | 状態が適切にリセット | コンテキスト喪失 |
| LC-B-01 | 1文字入力 | 境界値 | 正常に変換 | 最小入力 |
| LC-B-02 | 長文入力（100文字以上） | 境界値 | 正常に変換 | 大量入力 |

### 3.2 Zenzai変換

| Case ID | Input / Precondition | Perspective | Expected Result | Notes |
|---------|---------------------|-------------|-----------------|-------|
| ZN-N-01 | Zenzai有効 + "nihongo" | 正常系 | 高精度な変換候補 | 基本動作 |
| ZN-N-02 | プロファイル設定あり | 正常系 | 文脈に応じた変換 | パーソナライズ |
| ZN-A-01 | Zenzai無効 | 正常系 | 従来変換で動作 | フォールバック |
| ZN-A-02 | モデルファイル欠損 | 異常系 | エラーなく従来変換 | 障害耐性 |
| ZN-A-03 | CUDA非対応環境でCUDA選択 | 異常系 | Vulkan/CPUにフォールバック | 自動切替 |

### 3.3 候補選択

| Case ID | Input / Precondition | Perspective | Expected Result | Notes |
|---------|---------------------|-------------|-----------------|-------|
| CS-N-01 | 候補表示中に↓ | 正常系 | 次の候補を選択 | キー操作 |
| CS-N-02 | 候補表示中にEnter | 正常系 | 選択候補で確定 | 確定操作 |
| CS-N-03 | 候補表示中にEscape | 正常系 | 入力をキャンセル | キャンセル |
| CS-A-01 | 候補0件でSpace | 異常系 | エラーなし | 空候補 |
| CS-B-01 | 候補1件で↑ | 境界値 | 移動しない | 最小候補 |
| CS-B-02 | 最後の候補で↓ | 境界値 | 移動しない | 最大候補 |

### 3.4 入力モード切替

| Case ID | Input / Precondition | Perspective | Expected Result | Notes |
|---------|---------------------|-------------|-----------------|-------|
| IM-N-01 | かなモードで半角/全角 | 正常系 | 英字モードに切替 | 基本切替 |
| IM-N-02 | 入力中にモード切替 | 正常系 | 入力確定後切替 | 入力保護 |
| IM-A-01 | 高速連打でモード切替 | 異常系 | 状態が安定 | 連打耐性 |

### 3.5 ファンクションキー変換

| Case ID | Input / Precondition | Perspective | Expected Result | Notes |
|---------|---------------------|-------------|-----------------|-------|
| FK-N-01 | "aiu"入力後F6 | 正常系 | "あいう"に変換 | ひらがな |
| FK-N-02 | "aiu"入力後F7 | 正常系 | "アイウ"に変換 | カタカナ |
| FK-N-03 | "aiu"入力後F8 | 正常系 | "ｱｲｳ"に変換 | 半角カナ |
| FK-N-04 | "aiu"入力後F9 | 正常系 | "ａｉｕ"に変換 | 全角英数 |
| FK-N-05 | "aiu"入力後F10 | 正常系 | "aiu"に変換 | 半角英数 |
| FK-A-01 | 入力なしでF6 | 異常系 | 何も起きない | 空状態 |

---

## 4. テスト実行方法

### 4.1 単体テスト

#### Rust

```bash
cargo test --workspace
```

#### Swift

```bash
cd server-swift
swift test
```

#### Frontend

```bash
cd frontend
npm test
```

### 4.2 カバレッジ取得

#### Rust

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
```

### 4.3 手動テスト

1. ビルド: `cargo make build --debug`
2. IME登録: `regsvr32.exe build/azookey_windows.dll /s`
3. ランチャー起動: `build/launcher.exe` (管理者権限)
4. メモ帳等で入力テスト

---

## 5. 継続的インテグレーション

### 5.1 CI構成案

```yaml
# .github/workflows/ci.yml (案)
name: CI

on: [push, pull_request]

jobs:
  test-rust:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace

  test-swift:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: swift-actions/setup-swift@v1
      - run: cd server-swift && swift test

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: cd frontend && npm ci && npm test
```

---

## 6. 品質目標

### 6.1 カバレッジ目標

| コンポーネント | 目標 |
|---------------|------|
| server-swift | 80% |
| crates/* | 70% |
| frontend | 60% |

### 6.2 許容される不具合

| 重大度 | 定義 | 許容数 |
|--------|------|--------|
| Critical | IMEクラッシュ、データ損失 | 0 |
| High | 変換不能、候補表示不具合 | 0 |
| Medium | 特定環境での軽微な問題 | 3 |
| Low | 外観上の軽微な問題 | 10 |

---

## 7. テスト環境

### 7.1 推奨テスト環境

| 項目 | 推奨 |
|------|------|
| OS | Windows 10/11 (最新) |
| RAM | 8GB以上 |
| GPU | Vulkan対応GPU |
| 仮想マシン | 開発用に推奨 |

### 7.2 注意事項

- IMEのテストは仮想マシンまたは専用PCで実施推奨
- IMEクラッシュ時はWindowsがフリーズする可能性あり
- IME解除前に使用中アプリケーションを終了すること

---

**次のドキュメント**: [05-roadmap.md](./05-roadmap.md)

