//! quanttide-pi Rust 库
//!
//! Pi 智能体适配领域 Rust 语言工具包，表达 pi 主程序的数据约定：主程序拥有
//! agent 根目录与会话 JSONL 文件，不了解任何扩展；扩展数据一律位于根目录
//! 之下，其只读适配见 `crates/` 下的各扩展 crate——扩展以扩展方式单向依赖
//! 本 crate，对应 pi 主程序与扩展的关系。

use std::path::PathBuf;

/// 领域英文名
pub const DOMAIN: &str = "pi-agent";

/// 包版本（取自 `Cargo.toml`）
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 覆盖 agent 数据根目录的环境变量（pi 主程序约定）。
pub const AGENT_DIR_ENV: &str = "PI_CODING_AGENT_DIR";

/// 会话 JSONL 在 agent 根目录下的子目录。
pub const SESSIONS_DIR: &str = "sessions";

/// 解析 agent 数据根目录：`PI_CODING_AGENT_DIR` 优先，缺省为 `$HOME/.pi/agent`。
pub fn agent_root() -> PathBuf {
    let configured = std::env::var(AGENT_DIR_ENV).ok();
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    resolve_agent_root(configured.as_deref(), &home)
}

/// 会话 JSONL 目录（事件日志，权威会话事实）。
pub fn sessions_dir() -> PathBuf {
    agent_root().join(SESSIONS_DIR)
}

fn resolve_agent_root(configured: Option<&str>, home: &str) -> PathBuf {
    match configured.map(str::trim).filter(|s| !s.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(home).join(".pi").join("agent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_agent_root_default() {
        assert_eq!(
            resolve_agent_root(None, "/home/u"),
            PathBuf::from("/home/u/.pi/agent")
        );
    }

    #[test]
    fn test_resolve_agent_root_env_override() {
        assert_eq!(
            resolve_agent_root(Some("/custom/agent"), "/home/u"),
            PathBuf::from("/custom/agent")
        );
        assert_eq!(
            resolve_agent_root(Some("  "), "/home/u"),
            PathBuf::from("/home/u/.pi/agent")
        );
    }

    #[test]
    fn test_sessions_dir_below_agent_root() {
        let root = resolve_agent_root(None, "/home/u");
        assert_eq!(
            root.join(SESSIONS_DIR),
            PathBuf::from("/home/u/.pi/agent/sessions")
        );
    }
}
