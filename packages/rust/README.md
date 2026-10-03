# quanttide-pi Rust 库

Pi 智能体适配领域 Rust 语言工具包。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
quanttide-pi = "0.1.0"
```

## 使用

```rust
use quanttide_pi::{DOMAIN, VERSION};

println!("{DOMAIN} v{VERSION}");
```

读取 pi 会话与记忆（`pi-hermes-memory` 扩展包）：

```toml
[dependencies]
pi-hermes-memory = "0.1.0"
```

```rust
use pi_hermes_memory::{db_path, memory, session};

let sessions = session::list_sessions(&db_path(), 10)?;
let memories = memory::load_memories(&db_path())?;
```

## 结构

本包是主程序包 `quanttide-pi`，只表达 pi 主程序的数据约定（agent 根目录、会话 JSONL 目录），不感知扩展。扩展在 `crates/` 下独立成包，单向依赖本包——对应 pi 主程序与扩展的关系；当前有 `crates/pi-hermes-memory`（会话与记忆只读适配），新增扩展在 `crates/` 下建包即可。

## 测试

```bash
cargo test --workspace
```

## 许可

Apache-2.0
