# OverlayPlayer (for macOS)

macOS上で作業しながら動画（Amazon Prime Video）や音楽（YouTube Music等）を快適に視聴できる、常時最前面・クリックスルー対応のオーバーレイデスクトップアプリケーションです。

---

## 主な機能

- 🎬 **Amazon Prime Video 対応**: WKWebViewによるネイティブFairPlay DRM再生（フルHD対応）
- 🎵 **マルチサービス対応**: YouTube Music, YouTube, Twitch, 任意のカスタムURLへのワンクリック切り替え
- 📌 **常時最前面表示 (Always on Top)**: どのワークスペースや作業画面でも最前面にフローティング表示
- 👻 **クリックスルーモード (操作透過)**:
  - 動画を観ながら背面のテキストエディタやブラウザを直接クリック・操作可能
  - **ショートカット**: `Cmd + Shift + X` で瞬時にON/OFF切り替え
- 📐 **サイズプリセット**:
  - 小 (480x270) / 中 (720x405) / 大 (960x540)
  - ウィンドウの縁をドラッグして自由なサイズにリサイズ可能
- 🎛️ **コントロールバーの折りたたみ**: 動画のみを全画面表示し、ホバーで再展開可能
- ✨ **Prime Video UI最適化**: 余分なナビゲーションバーを縮小して動画コンテンツをすっきり表示

---

## 開発・起動方法

### 前提条件
- macOS
- Node.js (v20以上)
- Rust (`cargo` / `rustc`)

### 起動・ビルドコマンド
```bash
# 依存関係のインストール
npm install

# 開発モードで起動
npm run tauri dev

# アプリケーションのビルド (.app / .dmg)
npm run build:app
```

ビルド完了後、以下のディレクトリに macOS アプリケーションおよびインストーラー（DMG）が生成されます：
- **`.app`**: `src-tauri/target/release/bundle/macos/OverlayPlayer.app`
- **`.dmg`**: `src-tauri/target/release/bundle/dmg/OverlayPlayer_<version>_<arch>.dmg`

### CI / CD (GitHub Actions)
GitHub リポジトリへプッシュまたはリリースタグ（`v*`）を作成すると、GitHub Actions により自動で macOS 向けバイナリ（DMG / App）がビルドされます。
- プッシュ・PR時: 各実行画面の **Artifacts** からダウンロード可能
- リリースタグ時: **GitHub Releases** に自動公開

---

## 操作ショートカット
| ショートカット | 機能 |
| :--- | :--- |
| `Cmd + Shift + X` | クリックスルー（操作透過）のON / OFF切り替え |
| ウィンドウ上部中央ドラッグ | ウィンドウの移動 |
