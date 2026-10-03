//! `pi-hermes-memory` 扩展的数据访问适配。
//!
//! 对应 pi 扩展机制中的 `pi-hermes-memory` 包：主程序 crate [`quanttide_pi`]
//! 只提供数据约定（agent 根目录、会话 JSONL），本 crate 以扩展方式单向依赖
//! 主程序，读取该扩展维护的记忆 Markdown 与 `sessions.db` 检索索引，只读
//! 数据库侧的会话（[`session`]）与记忆（[`memory`]），共享打开逻辑见 [`db`]。

pub mod db;
pub mod memory;
pub mod session;

use std::path::PathBuf;

use quanttide_pi::agent_root;

/// 插件存储子目录（位于 agent 根目录之下）。
const PLUGIN_DIR: &str = "pi-hermes-memory";

/// 插件检索索引 `sessions.db` 的路径。
pub fn db_path() -> PathBuf {
    agent_root().join(PLUGIN_DIR).join("sessions.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path_ends_with_sessions_db() {
        assert!(db_path().ends_with("pi-hermes-memory/sessions.db"));
    }
}
