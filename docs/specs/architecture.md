# azooKey-Windows アーキテクチャ設計書

## 1. 全体アーキテクチャ

azooKey-Windowsは、以下の4つの主要コンポーネントから構成されるマルチプロセスアーキテクチャを採用しています。

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Windows OS                                    │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   アプリケーション                            │   │
│  │  (メモ帳, Word, Chrome など)                                 │   │
│  └────────────────────────────┬───────────────────────────────┘   │
│                               │ TSF (Text Services Framework)       │
│                               ▼                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │               IME Client (azookey_windows.dll)              │   │
│  │                    Rust + Windows API                         │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │   │
│  │  │ TSF Handler  │  │  Composition │  │   IPC Client    │   │   │
│  │  │ (キー入力)   │  │  (入力状態)  │  │   (gRPC)        │   │   │
│  │  └──────────────┘  └──────────────┘  └────────┬────────┘   │   │
│  └────────────────────────────────────────────────┼─────────────┘   │
│                                                   │                  │
│  ┌────────────────────────────────────────────────┼─────────────┐   │
│  │                 gRPC Server (azookey-server.exe)│            │   │
│  │                        Rust                     │            │   │
│  │  ┌──────────────────────────────────────────────▼──────────┐ │   │
│  │  │                    AzookeyService                       │ │   │
│  │  │  - AppendText    - MoveCursor    - SetContext          │ │   │
│  │  │  - RemoveText    - ShrinkText    - UpdateConfig        │ │   │
│  │  │  - ClearText                                            │ │   │
│  │  └──────────────────────────────────────────────────────────┘ │   │
│  │                              │ FFI                            │   │
│  │  ┌───────────────────────────▼──────────────────────────────┐ │   │
│  │  │            Conversion Engine (azookey-server.dll)        │ │   │
│  │  │                        Swift                              │ │   │
│  │  │  ┌────────────────────────────────────────────────────┐  │ │   │
│  │  │  │         AzooKeyKanaKanjiConverter                  │  │ │   │
│  │  │  │    (KanaKanjiConverterModule Swift Package)        │  │ │   │
│  │  │  └────────────────────────────────────────────────────┘  │ │   │
│  │  │  ┌──────────────────┐  ┌─────────────────────────────┐  │ │   │
│  │  │  │     Dictionary   │  │       Zenzai (Neural)       │  │ │   │
│  │  │  │  (辞書データ)    │  │  (GGUF モデル - llama.cpp)  │  │ │   │
│  │  │  └──────────────────┘  └─────────────────────────────┘  │ │   │
│  │  └──────────────────────────────────────────────────────────┘ │   │
│  └───────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐   │
│  │                  UI Process (ui.exe)                          │   │
│  │                   Rust + WebView (wry)                        │   │
│  │  ┌──────────────────────────────────────────────────────────┐│   │
│  │  │              候補ウィンドウ (WebView)                    ││   │
│  │  │    HTML/CSS/JavaScript によるレンダリング               ││   │
│  │  └──────────────────────────────────────────────────────────┘│   │
│  │  ┌──────────────────────────────────────────────────────────┐│   │
│  │  │                入力モードインジケータ                    ││   │
│  │  │               「あ」/ 「A」表示                         ││   │
│  │  └──────────────────────────────────────────────────────────┘│   │
│  └───────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐   │
│  │                Settings App (Tauri)                           │   │
│  │              frontend/ (React + TypeScript)                   │   │
│  │  ┌──────────────────────────────────────────────────────────┐│   │
│  │  │   一般設定 | 外観設定 | Zenzai設定 | バージョン情報     ││   │
│  │  └──────────────────────────────────────────────────────────┘│   │
│  └───────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

## 2. コンポーネント詳細

### 2.1 IME Client (`crates/client/`)

**役割**: Windows TSF (Text Services Framework) を利用してIMEとして動作するDLL

**主要モジュール**:

| ファイル | 役割 |
|----------|------|
| `tsf/text_service.rs` | TSF TextInputProcessor実装 |
| `tsf/key_event_sink.rs` | キー入力イベントハンドラ |
| `tsf/edit_session.rs` | テキスト編集セッション管理 |
| `tsf/display_attribute.rs` | 変換候補の表示属性 |
| `engine/composition.rs` | 入力状態管理（Composing/Previewing/Selecting） |
| `engine/ipc_service.rs` | gRPCクライアント |
| `engine/state.rs` | IME全体の状態管理 |

**選定理由**:
- TSFはWindows標準のIMEフレームワークで、すべてのWindows アプリケーションと互換性がある
- Rustを選択した理由: メモリ安全性、windows-rsクレートによる型安全なWindows API呼び出し

### 2.2 gRPC Server (`crates/server/`)

**役割**: IME ClientとConversion Engine間のブリッジ

**通信定義** (`crates/shared/service.proto`):

```protobuf
service AzookeyService {
  rpc AppendText (AppendTextRequest) returns (AppendTextResponse);
  rpc RemoveText (RemoveTextRequest) returns (RemoveTextResponse);
  rpc ShrinkText (ShrinkTextRequest) returns (ShrinkTextResponse);
  rpc MoveCursor (MoveCursorRequest) returns (MoveCursorResponse);
  rpc ClearText (ClearTextRequest) returns (ClearTextResponse);
  rpc SetContext (SetContextRequest) returns (SetContextResponse);
  rpc UpdateConfig (UpdateConfigRequest) returns (UpdateConfigResponse);
}
```

**選定理由**:
- gRPC: 効率的なバイナリプロトコル、型安全な通信
- プロセス分離: Swift製エンジンとRust製クライアントの分離が必要
- tonicクレート: RustのgRPC実装として成熟

### 2.3 Conversion Engine (`server-swift/`)

**役割**: かな漢字変換の実行

**依存関係**:
```swift
.package(url: "https://github.com/azookey/AzooKeyKanaKanjiConverter", branch: "7d5dd99")
```

**公開関数** (FFI経由):

| 関数 | 役割 |
|------|------|
| `Initialize` | 辞書・Zenzaiモデルの初期化 |
| `LoadConfig` | 設定ファイルの読み込み |
| `AppendText` | 文字追加とリアルタイム変換 |
| `RemoveText` | 文字削除 |
| `MoveCursor` | カーソル移動 |
| `ClearText` | 入力クリア |
| `GetComposedText` | 変換候補取得 |
| `ShrinkText` | 確定と縮小 |
| `SetContext` | 文脈設定（左側テキスト） |

**選定理由**:
- AzooKeyKanaKanjiConverterがSwiftで実装されている
- Swift for Windowsにより、Swiftコードをそのまま利用可能
- 変換ロジックの再実装を避け、オリジナルとの互換性を維持

### 2.4 UI Process (`crates/ui/`)

**役割**: 候補ウィンドウと入力モードインジケータの表示

**技術スタック**:
- `tao`: クロスプラットフォームウィンドウ管理
- `wry`: WebViewラッパー
- HTML/CSS/JavaScript: 候補リストのレンダリング

**ウィンドウ特性**:
```rust
WS_EX_TOOLWINDOW  // タスクバーに表示しない
WS_EX_NOACTIVATE  // フォーカスを奪わない
WS_EX_TOPMOST     // 常に最前面
WS_POPUP          // ポップアップウィンドウ
```

**選定理由**:
- WebViewによるUI: スタイリングの柔軟性、ダークモード対応の容易さ
- wry: Tauriで使用されている実績のあるWebViewラッパー

### 2.5 Settings App (`frontend/`)

**役割**: ユーザー設定GUI

**技術スタック**:
- Tauri: Rust + Web技術によるデスクトップアプリ
- React + TypeScript
- shadcn/ui: UIコンポーネント

**設定画面**:
- 一般設定（バージョン情報）
- 外観設定（テーマ）
- Zenzai設定（有効化、プロファイル、バックエンド選択）

**設定ファイル**: `%APPDATA%/Azookey/settings.json`
```json
{
    "version": "0.0.1",
    "zenzai": {
        "enable": false,
        "profile": "",
        "backend": "cpu"
    }
}
```

## 3. プロセス間通信

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   IME Client                    gRPC Server                        │
│  (DLL in App)                   (Standalone)                       │
│        │                              │                             │
│        │  AppendText("a")            │                             │
│        │────────────────────────────▶│                             │
│        │                              │   FFI Call                  │
│        │                              │──────────▶ Swift Engine     │
│        │                              │◀──────────                  │
│        │  Response(candidates)        │                             │
│        │◀────────────────────────────│                             │
│        │                              │                             │
│        ▼                              │                             │
│   UI Process                          │                             │
│  (Standalone)                         │                             │
│        │                              │                             │
│        │  window.proto IPC            │                             │
│        │◀─────────────────────────────│                             │
│        │  (候補リスト更新)            │                             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 4. ビルド構成

### 4.1 ビルドターゲット

| ターゲット | 成果物 | 説明 |
|-----------|--------|------|
| x64 | `azookey_windows.dll` | 64bit IMEモジュール |
| x86 | `x86/azookey_windows.dll` | 32bit IMEモジュール（WOW64対応） |
| Swift | `azookey-server.dll` | 変換エンジン |
| UI | `ui.exe` | 候補ウィンドウ |
| Server | `azookey-server.exe` | gRPCサーバー |
| Launcher | `launcher.exe` | 起動ヘルパー |
| Settings | `azookey-settings.exe` | 設定アプリ (Tauri) |

### 4.2 ビルドコマンド

```bash
cargo install --force cargo-make
cargo make build --release
```

### 4.3 依存リソース

| リソース | パス | 説明 |
|----------|------|------|
| 辞書データ | `Dictionary/` | かな漢字変換辞書 |
| 絵文字辞書 | `EmojiDictionary/` | 絵文字変換データ |
| Zenzaiモデル | `zenz.gguf` | ニューラル変換モデル (GGUF形式) |
| llama.cpp | `llama_*/` | CPU/CUDA/Vulkan用推論ライブラリ |
| Swift Runtime | `*.dll` | Swift実行時ライブラリ |

## 5. インストール構成

### 5.1 インストーラ

Inno Setupを使用したインストーラ (`installer/Installer.iss`)

### 5.2 レジストリ登録

IMEの登録:
```
regsvr32.exe "path/to/azookey_windows.dll" /s
regsvr32.exe "path/to/x86/azookey_windows.dll" /s
```

### 5.3 スタートアップ登録

`launcher.exe` をスタートアップに登録して、gRPCサーバーとUIプロセスを起動

---

**次のドキュメント**: [02-features.md](./02-features.md)

