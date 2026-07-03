# ichika — 项目状态与计划 (PLAN)

> 本文件由自动化扫描于 **2026-07-04** 生成，记录项目当前状态、近期进展与后续计划。

## 1. 项目概述

- **名称**：`ichika`
- **简介**：基于 flume 的线程池自动构造辅助库。
- **远程仓库**：git@github.com:celestia-island/ichika.git
- **技术栈**：Rust
- **类别**：rust-lib

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：干净
- **最近提交时间**：2026-07-03
- **最近提交**：fix(quality): re-export num_cpus, fix retry_with error type, de-flake timing-sensitive tests
- **分支对比**：`dev` 领先 `master` 44 个提交

## 3. 未提交改动

无。

## 4. 近期进展（最近提交）

- fix(quality): re-export num_cpus, fix retry_with error type, de-flake timing-sensitive tests
- fix(macros): stop swallowing first-closure constraint as global
- fix(pool): honor min_threads floor when max_threads is also set
- fix(quality): propagate proc-macro panics as Result/compile_error, re-export log, drop unused deps
- fix(ci): correct self-referencing path in test.yml workflow trigger
- ci: switch docs workflow to lagrange

## 5. 后续计划

1. 完善文档示例与 `crates.io` 发布元数据（rust-version / metadata / docs.rs badge）。
2. 补充单元/集成测试，保持 `just test` 与 clippy `-D warnings` 通过。
3. 定期刷新本 PLAN.md 以反映最新状态。

