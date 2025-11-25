# azooKey-Windows 設計仕様書

このディレクトリには、azooKey-Windows プロジェクトの設計仕様書が格納されています。

---

## ドキュメント一覧

### 全体設計

| ドキュメント | 内容 |
|-------------|------|
| [overview.md](./overview.md) | プロジェクト概要、背景、関連プロジェクト |
| [architecture.md](./architecture.md) | システムアーキテクチャ、コンポーネント構成 |
| [technical-decisions.md](./technical-decisions.md) | 技術選定理由、ライブラリ選択の根拠 |
| [testing-strategy.md](./testing-strategy.md) | テスト戦略、テスト観点表 |
| [roadmap.md](./roadmap.md) | 開発ロードマップ、TODO |

### 機能仕様

| ドキュメント | 機能 | 状態 |
|-------------|------|:----:|
| [features.md](./features.md) | 機能一覧（実装済み・未実装） | - |
| [learning-history.md](./learning-history.md) | 履歴学習機能 | ✅ 実装済み |
| [user-dictionary.md](./user-dictionary.md) | ユーザー辞書機能 | ✅ 実装済み |
| prediction.md | 予測変換 | 📝 予定 |
| magic-conversion.md | いい感じ変換 | 📝 予定 |
| theme.md | テーマ機能 | 📝 予定 |

状態: ✅ 実装済み / 🚧 設計中 / 📝 予定

---

## 読む順序

1. **概要を把握したい場合**: `overview.md` → `architecture.md`
2. **機能を確認したい場合**: `features.md` → 各機能仕様書
3. **技術的な理解を深めたい場合**: `technical-decisions.md`
4. **テストを追加したい場合**: `testing-strategy.md`
5. **開発に参加したい場合**: `roadmap.md`

---

## 関連リンク

### 本家 azooKey

- [azooKey 公式サイト](https://azookey.com/)
- [azooKey iOS/iPadOS (GitHub)](https://github.com/azooKey/azooKey)
- [azooKey-Desktop macOS (GitHub)](https://github.com/azooKey/azooKey-Desktop)
- [AzooKeyKanaKanjiConverter (GitHub)](https://github.com/azooKey/AzooKeyKanaKanjiConverter)

### コミュニティ

- [azooKey Discord](https://discord.gg/dY9gHuyZN5)
- [GitHub Issues](https://github.com/fkunn1326/azooKey-Windows/issues)

---

## 更新履歴

| 日付 | 内容 |
|------|------|
| 2025-11-26 | 初版作成、docs/specs/ に移動 |
