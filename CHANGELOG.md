# Changelog

## [Unreleased]

- 初始化工具箱仓库：预留 `packages/` 语言包目录。
- 初始化 Rust 库 `packages/rust`（`quanttide-pi` 0.1.0）：领域常量、版本导出与集成测试。
- 新增 GitHub Action `.github/workflows/release-rust.yml`：`rust/*` 标签触发，质量门（元数据校验、fmt、test、clippy、pack 试打包）→ crates.io 发布 → 补 GitHub Release。
