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

## 测试

```bash
cargo test
```

## 许可

Apache-2.0
