# azooKey-Windows プロジェクト概要

## 1. プロジェクトの目的

azooKey-WindowsはiOS/macOS向けに開発された日本語入力システム「azooKey」のWindows移植版です。オリジナル版と同等の高品質なかな漢字変換体験をWindowsユーザーに提供することを目的としています。

### 1.1 背景

- **azooKey**: [ensan-hcl(三輪敬太)](https://github.com/ensan-hcl)氏によって開発されたOSSの日本語入力キーボードアプリ
- **未踏IT事業**: 2024年度の未踏IT事業で「ニューラル言語モデルによる個人最適な日本語入力システムの開発」として採択
- **Zenzai**: ニューラルかな漢字変換エンジン。Transformer Decoderベースの言語モデルを利用し高精度な変換を実現
- **AzooKeyKanaKanjiConverter**: Swiftで実装されたかな漢字変換エンジン（Swift Package）

### 1.2 Windows移植

[fkunn1326](https://github.com/fkunn1326)氏により、AzooKeyKanaKanjiConverterをWindows上で動作させることで移植が実現されました。

## 2. 関連プロジェクト

| プロジェクト | URL | 説明 |
|-------------|-----|------|
| azooKey (iOS) | https://github.com/azooKey/azooKey | iOS/iPadOS向けキーボードアプリ |
| azooKey-Desktop (macOS) | https://github.com/azooKey/azooKey-Desktop | macOS向け日本語入力システム |
| AzooKeyKanaKanjiConverter | https://github.com/azooKey/AzooKeyKanaKanjiConverter | かな漢字変換エンジン (Swift Package) |
| fcitx5-hazkey (Linux) | https://github.com/7ka-Hiira/fcitx5-hazkey | Linux向けクライアント実装 |
| azooKey-Windows | https://github.com/fkunn1326/azooKey-Windows | **本プロジェクト** |

## 3. 主要参考資料

### 3.1 公式ドキュメント・記事

- [azooKey公式サイト](https://azookey.com/)
- [オープンソース情報](https://azookey.com/OpenSource)
- [Zennブログ - azooKey開発近況（2025年6月）](https://zenn.dev/azookey/articles/7a2c2d20a3cc4a)
- [Zennブログ - Foundation Models Frameworkで絵文字を推薦](https://zenn.dev/azookey/articles/153b1bf4da1119)
- [未踏IT事業プロジェクト概要](https://www.ipa.go.jp/jinzai/mitou/it/2024/gaiyou-ok-3.html)

### 3.2 IME開発参考資料

- [khiin-rs (Windows TSF実装)](https://github.com/OMAMA-Taioan/khiin-rs/tree/master/windows)
- [Google Mozc (TSF実装)](https://github.com/google/mozc/tree/master/src/win32/tip)
- [Microsoft Windows TSF サンプル](https://github.com/microsoft/Windows-classic-samples/tree/main/Samples/Win7Samples/winui/input/tsf/textservice)
- [ajemi (Rust TSF実装)](https://github.com/dec32/ajemi)

## 4. ライセンス

MIT License

## 5. 開発支援

- [GitHub Sponsors (Miwa)](https://github.com/sponsors/ensan-hcl): 変換エンジンの開発者
- [Patreon (fkunn1326)](https://www.patreon.com/c/fkunn1326): Windows移植担当者

---

**次のドキュメント**: [01-architecture.md](./01-architecture.md)

