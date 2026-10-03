# quanttide-pi-toolkit

量潮 Pi 智能体工具箱

## 定位

`quanttide-pi`（适配轴）的共享库与工具集：把 Pi 智能体的接入能力沉淀为可复用的包，供适配器与各端调用。

- 归属：适配轴 `adapters/quanttide-pi` 的 `packages/quanttide-pi-toolkit`
- 聚合：能力轴 `assets/quanttide-toolkit` 的 `adapters/quanttide-pi-toolkit`

## 包清单

按语言分置于 `packages/`：

| 语言 | 包 | 版本 | 说明 |
|:--|:--|:--|:--|
| Rust | [`quanttide-pi`](packages/rust) | 0.1.0 | 主程序包：pi 主程序数据约定，工作区入口 |
| Rust | [`pi-hermes-memory`](packages/rust/crates/pi-hermes-memory) | 0.1.0 | 扩展包：`pi-hermes-memory` 会话与记忆只读适配 |

## 许可

[Apache-2.0](LICENSE)
