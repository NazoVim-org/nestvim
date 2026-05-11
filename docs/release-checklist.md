# Release Checklist

リリース前に以下を**必ず**完了すること。

## 必須チェック

- [ ] `cargo fmt --check`
- [ ] `cargo clippy`
- [ ] `cargo test`
- [ ] `cargo run -- --help`
- [ ] バージョン表記確認（`Cargo.toml` / CLI表示 / リリースタグの整合）
- [ ] 対応OS確認（サポート対象OSでの動作確認）
