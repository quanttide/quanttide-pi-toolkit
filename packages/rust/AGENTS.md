# AGENTS.md - quanttide-pi Rust 库

## 项目结构

```
packages/rust/
├── AGENTS.md        # 本文件
├── CHANGELOG.md     # 版本变更记录
├── Cargo.toml       # 工作区与主程序包 quanttide-pi
├── README.md        # 项目说明
├── src/
│   └── lib.rs       # 主程序库入口与数据约定（agent 根目录、会话 JSONL）
├── crates/
│   └── pi-hermes-memory/  # 扩展包：sessions.db 会话与记忆只读适配
└── tests/
    └── package.rs   # 集成测试
```

## 扩展关系

主程序包只表达 pi 主程序的数据约定，不感知任何扩展；扩展在 `crates/` 下独立成包，单向依赖 `quanttide-pi`——对应 pi 主程序与扩展（`settings.json` 的 `packages`）的关系。新增扩展在 `crates/` 下建包即可，`members = ["crates/*"]` 自动纳入工作区。

## 事实源

Pi 智能体的接入方式与语境以 `adapters/quanttide-pi` 为准；本库只做表达，不定义事实。

## 提交约定

Conventional Commits（`feat:` / `fix:` / `docs:` / `chore:`）；破坏性变更标 `!` 并在 body 说明迁移方式。

## 测试

```bash
cargo test --workspace
```
