# 长期记忆 — lonely-coder 项目

## 项目结构概况

仓库包含三个历史层：
1. `archive/` — 2013-2015年 Wuma/无码 PHP 社交平台（已废弃，含严重安全漏洞）
2. `analysis-report.md` — 2026年4月 PHP 代码安全审计报告（17严重+18高危+16中危）
3. `pair-terminal/` — 2026年5月至今的 Rust 终端结对编程工具（**活跃项目**）
4. `product-design/` — 2026年8月创建的 DevCave 产品设计文档集

---

## pair-terminal 技术状态（最后评估：2026-08-02）

- 架构：3-crate workspace（pair-common / pair-server / pair-client）
- 代码量：~3,530 行 Rust 源码
- 测试：36 个测试全部通过（23 common + 13 server），pair-client 无测试
- 构建/Clippy/格式化：均通过
- CI 配置：`.github/workflows/ci.yml` 含 fmt/clippy/build/test/rustsec

修复过的问题：
- `pair-server/src/ws_handler.rs` 未使用变量 `tid_clone`（已移除）
- 代码格式化（已执行 `cargo fmt --all`）

遗留问题：
- pair-client 无任何测试
- Login/Profile/Leaderboard/Upload 为 stub 未实现
- P2P/WebRTC 标志存在但无实现
- 无端到端 WebSocket 集成测试

---

## DevCave 产品设计（2026-08-02 创建）

**产品定位**：程序员专属社交平台，五大模块：技术分享、协作匹配、匿名树洞、技术问答、技术活动

**文件位置**：`/product-design/`（共 9 个 Markdown 文件）

**设计规范**：
- 主色：`#7C3AED`（紫色）
- 背景：`#0D0D0D` / `#141414` / `#1E1E1E` / `#2A2A2A`
- 字体：Geist Mono（代码）/ Inter（正文）
- 风格：极简主义，深色优先，开发者美学

**产品名称**：DevCave（开发者的据点）

---

## 用户偏好与约定

- 评估类任务：用户期望同时修复发现的问题（不只是报告）
- 文档类任务：输出 Markdown，存放在项目仓库内
- 语言：中文回复
