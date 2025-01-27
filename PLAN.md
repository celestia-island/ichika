# ichika — 项目状态与计划 (PLAN)

> 本文件由自动化扫描于 **2026-07-04** 生成，记录项目当前状态、近期进展与后续计划。

## Refresh log 2026-07-14

- **当前分支**：`dev` · 领先 `origin/dev` 0 commits · 工作区干净
- **最近提交**：`🔧 Pin script recipes to the resolved Git Bash to survive WSL shadowing.` (`f3f2c9c`)
- **未提交改动**：无
- **后续动作**：
  1. 验证 `🔧 Pin script recipes` 提交后，ichika workspace（`ichika` / `ichika-macros`）的 just 配方在 WSL 环境下使用解析到的 Git Bash 路径（而非被 WSL 影线的 shim）能稳定跑通 `just lint` / `just test`；如 macros 宏展开路径仍报 Git 相关错误，续在 `dev` 分支上补丁。
  2. 跟进跨仓 `[patch]` 收敛到 `~/.cargo/config.toml`（见 `entelecheia/PLAN.md` §6 跨仓依赖约定）后，ichika workspace `Cargo.toml` 中对 entelecheia / hifumi 等生态内 crate 的内联 `[patch.*]` 是否需移除。
  3. 在顶层 `patches/` 长期方案中登记 ichika 线程池/宏展开相关的 flume 上游兼容检查点（避免 flume 升级时无声破坏）。
- **跨仓依赖**：上游依据 → `entelecheia/PLAN.md`；ichika 是 entelecheia 生态中负责线程池 / 宏的子仓，与 hifumi（构建脚本）、malkuth（leader/lease 同属并发原语）等 sibling 仓共享 workspace 风格。

## 1. 项目概述

- **名称**：`ichika`
- **简介**：基于 flume 的线程池自动构造辅助库。
- **远程仓库**：<git@github.com>:celestia-island/ichika.git
- **技术栈**：Rust（workspace：`ichika` / `ichika-macros`）
- **类别**：rust-lib

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：见 `git status`
- **最近提交时间**：2026-07-04
- **工具链**：stable（`rust-toolchain.toml`），`rust-version = "1.85"`（workspace 统一）
- **分支对比**：`dev` 领先 `master` 若干提交

## 3. 发布元数据（crates.io / docs.rs）

- [x] `rust-version = "1.85"`（workspace.package 统一设置，`ichika` / `ichika-macros` 均已继承）
- [x] `keywords` / `categories`（主包 `ichika`）
- [x] `[package.metadata.docs.rs]` `all-features = true`（主包 `ichika`）
- [x] README docs.rs badge
- [x] README LICENSE 引用修正为 `./LICENSE`

## 4. 文档

- [x] README Quick Start 示例已重写，语法正确、与实际 API（`pipe!` / `ThreadPool::{send,recv}`）一致，已用 `cargo check --example` 验证（同步 + 异步两段示例）。

## 5. 开发工具

- [x] `justfile`（`test` / `clippy -D warnings` / `check` / `fmt` / `build` / `ci`），与兄弟库 seia/kou/arona 风格一致。

## 6. 验证（本地 gate）

- `cargo check --workspace --all-targets --all-features` ✅
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✅
- `cargo fmt --all -- --check` ✅
- `cargo test --workspace --all-features` ✅（集成测试全部通过）
- `just fmt-check` / `just clippy` ✅

## 7. 近期进展（最近提交）

- docs: fix README example errors
- build: add justfile
- chore: complete crates.io / docs.rs metadata (rust-version, keywords, categories)
- docs: add PLAN.md current-status snapshot
- fix(quality): re-export num_cpus, fix retry_with error type, de-flake timing-sensitive tests
- fix(macros): stop swallowing first-closure constraint as global
- fix(pool): honor min_threads floor when max_threads is also set

## 8. 后续计划

1. 补充单元/集成测试覆盖边界场景，保持 `just test` 与 clippy `-D warnings` 通过。
2. 视发布需要补充 `CHANGELOG`、crates.io 长描述等。
3. 定期刷新本 PLAN.md 以反映最新状态。
