# Changelog

## [Unreleased]

- 初始化工具箱仓库：预留 `packages/` 语言包目录。
- 初始化 Rust 库 `packages/rust`（`quanttide-pi` 0.1.0）：领域常量、版本导出与集成测试。
- Rust 包 `quanttide-pi` 新增 pi 主程序/插件分层数据访问（`core` + `plugins::hermes_memory`）。
- 新增 GitHub Action `.github/workflows/release-rust.yml`：`rust/*` 标签触发，质量门（元数据校验、fmt、test、clippy、pack 试打包）→ crates.io 发布 → 补 GitHub Release。
- Rust 包重构为工作区：`plugins::hermes_memory` 拆为独立扩展包 `crates/pi-hermes-memory`，单向依赖主程序包 `quanttide-pi`（对应 pi 主程序与扩展的关系）。
