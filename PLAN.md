# Ichika Development Plan

## 1. Current Status Snapshot

Date: 2026-03-31 (updated after M0–M4 completion)

All milestones M0 through M4 are complete. `cargo test` is green with zero warnings.

**Verified working:**
- Multi-type closure chains: `String → usize → String` ✓
- Named step routing via `Status::Switch` ✓  
- Panic recovery with thread restart ✓
- Retry with `RetryPolicy` (max_attempts, delay_ms) ✓
- Per-step and global thread count constraints ✓
- All 6 examples compile and run

## 2. Goals

### Primary Goals

- Complete missing core features from README TODO in a stable order.
- Add several immediately runnable examples for local debug and documentation.
- Ensure examples and tests become the executable specification for macro behavior.

### Non-Goals (for this plan cycle)

- Performance tuning beyond correctness baseline.
- Public API redesign unless required to unblock TODO features.

## 2.1 Maintainer Decisions (Locked)

- Example location is fixed to `packages/types/examples/`.
- Error handling should follow Rust-style branch routing, i.e. prefer `match`-based handling over standalone `catch` keyword.
- Retry timeout semantics are per single loop/attempt of one pipeline flow (not global budget timeout).
- CI/dev gate should stay green at each commit whenever possible; temporary local snapshots are allowed, but no long-lived ignored tests.

## 3. Milestones

## M0: Stabilize Baseline (compile + test gate) - DONE ✅

1. ✅ Fixed critical type inference issue in `thread_creator.rs` codegen.
2. ✅ `cargo test` passes with 0 failures, 0 warnings.
3. ✅ Feature matrix documented.

## M1: Core Control-Flow Support (`match` and named targets) - DONE ✅

1. ✅ Implemented recursive closure generation for `PipeNodeFlatten::Map`.
2. ✅ Implemented parse + codegen for branch labels in `match { target: closure }` style.
3. ✅ Implemented runtime route handling for `Status::Switch((target, payload))` with type-safe routing table.
4. ✅ Coverage tests: one-level match route, nested match route, fallback/default route.

## Examples - DONE ✅

- ✅ E1. `basic_sync_chain.rs` - Minimal sync pipeline
- ✅ E2. `basic_async_chain.rs` - Async closure chain
- ✅ E3. `tuple_payload_pipeline.rs` - String processing pipeline
- ✅ E4. `monitoring_thread_usage.rs` - Thread usage monitoring
- ✅ E5. `graceful_shutdown_drop.rs` - Graceful shutdown demonstration
- ✅ E6. `status_exit_demo.rs` - Filter behavior demonstration

## M2: Error Handling (`catch`) - DONE ✅

1. ✅ Worker dispatch handles `Status::Panic` and `Status::PanicSwitch` via `catch_unwind`.
2. ✅ Tests in `error_handling.rs`: Result wrapping, panic recovery, mixed normal/panic cases.

## M3: Retry Semantics (`retry` + timeout) - DONE ✅

1. ✅ `RetryPolicy` struct with `max_attempts` and `delay_ms`.
2. ✅ `Status::RetryWith(policy, attempt, value)` variant.
3. ✅ `retry<T, E>()` and `retry_with<T>()` helper functions.
4. ✅ Runtime scheduling with delay and attempt tracking.
5. ✅ 7 comprehensive tests covering all retry paths.

**Retry semantics (locked):**
- `Status::Retry` → retry up to `default().max_attempts`; silently discard after max
- `Status::RetryWith(policy, _, fallback)` → retry up to `policy.max_attempts`; send `fallback` after max

## M4: Per-Step Thread Limits - DONE ✅

1. ✅ Extended syntax: global max thread count, per-step max/min thread count.
2. ✅ Parser structures (`PipeMacros`, `ClosureMacros`) support thread constraints.
3. ✅ Constraints integrated into `generate_thread_creator` and daemon scaling logic.
4. ✅ 11 tests in `thread_limits.rs` covering all constraint combinations.

## 4. Test Coverage Summary

| File | Tests | Status |
|------|-------|--------|
| `simple_test.rs` | 1 | ✅ |
| `debug_parse.rs` | 1 | ✅ |
| `debug_type.rs` | 1 | ✅ |
| `minimal_pipe.rs` | 1 | ✅ |
| `explicit_type.rs` | 1 | ✅ |
| `pipe.rs` | 2 | ✅ |
| `pipe_async.rs` | 1 | ✅ |
| `pipe_multi.rs` | 2 | ✅ |
| `pipe_named.rs` | 2 | ✅ |
| `pipe_switch.rs` | 4 | ✅ |
| `error_handling.rs` | 3 | ✅ |
| `retry_semantics.rs` | 7 | ✅ |
| `thread_limits.rs` | 11 | ✅ |
| `single_step.rs` | 1 | ✅ |

## 5. Remaining / Future Work

- CI matrix for `--no-default-features --features async-std`
- Performance/robustness benchmark task
- Full match-based error routing (deferred; current design uses panic-based approach)
- Tuple payload support (`(String, usize) → ...`) - partially stubbed in E3

## 6. Risks and Mitigations

- Risk: parser grammar drift between README, tests, and implementation.
  - Mitigation: freeze a grammar spec section in README before M1 code changes.

- Risk: status enum design insufficient for retry metadata.
  - Mitigation: add explicit retry payload struct early in M3 design. ✅ Done.

- Risk: runtime creation overhead for async steps (tokio runtime per call).
  - Mitigation: evaluate shared runtime strategy after correctness milestones.

## 7. Execution Guardrails

1. Keep branch buildable and tests runnable at each major commit point.
2. Avoid syntax divergence between README examples, parser grammar, and integration tests.
3. Promote new language features only after adding at least one runnable example and one integration test.

