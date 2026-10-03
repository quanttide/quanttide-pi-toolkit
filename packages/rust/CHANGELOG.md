# Changelog

## [Unreleased]

- 初始化 Rust 库骨架：领域常量与版本导出。
- 新增主程序/插件分层：`core`（agent 根目录解析、会话 JSONL 目录）与 `plugins::hermes_memory`（sessions.db 会话列表、JSONL 导出、统计分析、记忆导出），提炼自 thera 的 `thera-pi`，去除项目耦合。
- 重构为工作区：`plugins::hermes_memory` 拆为独立扩展包 `crates/pi-hermes-memory`，单向依赖主程序包 `quanttide-pi`（对应 pi 主程序与扩展的关系），主程序包卸下外部依赖只留 `core` 数据约定。
- 拆平主程序包 `src/core/`：数据约定上提至 crate 根（`quanttide_pi::agent_root` 等），`quanttide_pi::core` 路径不再存在。
