# pi-hermes-memory（Rust）

`pi-hermes-memory` 扩展的只读数据访问适配：会话列表与 JSONL 导出、统计分析、记忆导出。

对应 pi 扩展机制中的 `pi-hermes-memory` 包，以扩展方式单向依赖主程序 crate [`quanttide-pi`](../..)，主程序不感知本 crate。

## 使用

```toml
[dependencies]
pi-hermes-memory = "0.1.0"
```

```rust
use pi_hermes_memory::{db_path, memory, session};

let sessions = session::list_sessions(&db_path(), 10)?;
let memories = memory::load_memories(&db_path())?;
```

## 测试

```bash
cargo test -p pi-hermes-memory
```

## 许可

Apache-2.0
