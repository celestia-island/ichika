# Ichika Development Plan

## 1. Current Status Snapshot

Date: 2026-03-31

Based on source and `cargo test -q` verification:

- Workspace root is a virtual manifest (`Cargo.toml` has `[workspace]` only), so runnable examples should be placed under crate package path (`packages/types/examples/`) rather than workspace root `examples/`.
- `pipe!` currently supports basic closure chain (sync + async closure parsing and codegen).
- Named step syntax is parsed and rewritten, but `match` branch code generation is not implemented yet.
- Runtime status variants exist in `Status` (`Switch`, `Retry`, `Exit`, etc.), but daemon/worker logic currently only handles `Status::Next`.
- Existing tests indicate gaps:
  - `pipe_multi.rs`: parser does not support `catch` keyword or `target_a:` branch label syntax.
  - `pipe_named.rs`: macro panics at unimplemented recursive closure generation for `Map` node.

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

## M0: Stabilize Baseline (compile + test gate)

1. Fix/align parser grammar with currently desired syntax subset.
2. Ensure `cargo test` passes continuously for supported features; do not rely on `#[ignore]` as a transition strategy.
3. Define feature matrix in README/PLAN for:
   - supported now
   - in progress
   - planned

Deliverable:

- Clean baseline branch where unsupported syntax does not fail unexpectedly.

## M1: Core Control-Flow Support (`match` and named targets)

1. Implement recursive closure generation for `PipeNodeFlatten::Map`.
2. Implement parse + codegen for branch labels in `match { target: closure }` style.
3. Implement runtime route handling for `Status::Switch((target, payload))` (or equivalent final syntax).
4. Add coverage tests:
   - one-level match route
   - nested match route
   - fallback/default route

Deliverable:

- `pipe_named` and `pipe_multi` route-related scenarios passing.

## M2: Error Handling (`catch`)

1. Implement branch-like target under `match` for error path (Rust-style), replacing standalone `catch` syntax direction.
2. Extend parser to build error-path nodes.
3. Extend worker dispatch to handle `Status::Panic` and `Status::PanicSwitch`.
4. Add tests for:
   - recoverable error to normal output
   - unrecoverable error propagation

Deliverable:

- deterministic error handling path with clear docs.

## M3: Retry Semantics (`retry` + timeout)

1. Define retry policy model:
   - max attempts
   - delay/backoff
   - timeout/cancel (per single retry loop/attempt)
2. Add status payload support if needed (`Retry` currently has no metadata).
3. Implement scheduling behavior in daemon loop.
4. Add tests for retry success/failure/timeout.

Deliverable:

- retry behavior predictable and observable.

## M4: Per-Step Thread Limit

1. Extend syntax (already hinted by TODO comments):
   - global max thread count
   - per-step max/min thread count
2. Update parser structures (`PipeMacros`, `ClosureMacros`) for thread constraints.
3. Integrate constraints into `generate_thread_creator` and daemon scaling logic.
4. Add stress tests for fairness and starvation prevention.

Deliverable:

- thread usage can be configured per step with test coverage.

## 4. Immediate Examples Plan (do not execute yet)

Target location:

- `packages/types/examples/`

Run pattern:

- `cargo run -p ichika --example <example_name>`

### E1. `basic_sync_chain.rs`

Purpose:

- Minimal sync pipeline, no branch/error handling.

Flow:

- `String -> usize -> String`

Validation:

- send N requests, collect outputs, assert deterministic mapping.

Dependencies:

- none beyond existing crate deps.

### E2. `basic_async_chain.rs`

Purpose:

- Async closure chain under default tokio feature.

Flow:

- async step compute/transform, final output collect.

Validation:

- successful send/recv loop; no panic on runtime path.

Dependencies:

- existing tokio feature only.

### E3. `tuple_payload_pipeline.rs`

Purpose:

- Multi-argument tuple input/output pipeline.

Flow:

- `(String, usize) -> (String, usize, bool) -> String`

Validation:

- verify tuple destructuring and reconstruction.

Dependencies:

- none extra.

### E4. `monitoring_thread_usage.rs`

Purpose:

- Demonstrate `thread_usage()` and `task_count()` observability.

Flow:

- burst send tasks, periodically print pool metrics.

Validation:

- metrics change over time and return to idle baseline.

Dependencies:

- none extra.

### E5. `graceful_shutdown_drop.rs`

Purpose:

- Show pool drop behavior and shutdown semantics.

Flow:

- submit tasks, let scope end, confirm no hang.

Validation:

- process exits cleanly.

Dependencies:

- none extra.

### E6. `status_exit_demo.rs`

Purpose:

- If current parser/runtime allows: demonstrate `Status::Exit` path.

Flow:

- task returns exit status for subset input.

Validation:

- expected output count and non-blocking completion.

Dependencies:

- depends on final status dispatch behavior.

## 5. Test Plan Alignment

For each milestone:

1. Unit tests for parser AST shape (`packages/macros`).
2. Integration tests for macro expansion behaviors (`packages/types/tests`).
3. Example smoke test script:
   - run all examples in `packages/types/examples/` sequentially in CI.
4. Add CI matrix for features:
   - default (tokio)
   - `--no-default-features --features async-std`

## 6. Suggested Execution Order

1. M0 baseline stabilization.
2. M1 match + named routing.
3. Add E1-E4 examples immediately after M1 baseline is stable.
4. M2 catch.
5. Add/upgrade route+error examples (new E7 optional).
6. M3 retry.
7. M4 per-step thread limits.
8. Add performance/robustness benchmark task.

## 7. Risks and Mitigations

- Risk: parser grammar drift between README, tests, and implementation.
  - Mitigation: freeze a grammar spec section in README before M1 code changes.

- Risk: status enum design insufficient for retry metadata.
  - Mitigation: add explicit retry payload struct early in M3 design.

- Risk: runtime creation overhead for async steps (tokio runtime per call).
  - Mitigation: evaluate shared runtime strategy after correctness milestones.

## 8. Execution Guardrails

1. Keep branch buildable and tests runnable at each major commit point.
2. Avoid syntax divergence between README examples, parser grammar, and integration tests.
3. Promote new language features only after adding at least one runnable example and one integration test.
