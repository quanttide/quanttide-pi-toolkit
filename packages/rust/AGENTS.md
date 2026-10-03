# AGENTS.md - quanttide-pi Rust 库

## 项目结构

```
packages/rust/
├── AGENTS.md       # 本文件
├── CHANGELOG.md    # 版本变更记录
├── Cargo.toml      # Rust 包配置
├── README.md       # 项目说明
├── src/
│   └── lib.rs      # 库入口
└── tests/
    └── package.rs  # 集成测试
```

## 事实源

Pi 智能体的接入方式与语境以 `adapters/quanttide-pi` 为准；本库只做表达，不定义事实。

## 提交约定

Conventional Commits（`feat:` / `fix:` / `docs:` / `chore:`）；破坏性变更标 `!` 并在 body 说明迁移方式。

## 测试

```bash
cargo test
```
