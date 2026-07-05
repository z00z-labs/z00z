# 067 Glossary Claims

Phase 067 uses one live planner model only: deterministic replicated local
recomputation over canonical planner config and route-table digest. A separate
planner primary or secondary HA service is not implemented in this phase and
must remain non-live until it has durable state, activation boundaries, and
failover tests.

| term | code owner | artifact/API | positive test | negative test | claim level | evidence refs | plan id |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `deterministic replicated planner` | `crates/z00z_runtime/aggregators` | `PlannerAuthority`, `BatchPlanner`, `scenario_11/quorum/route_plan_report.json` | `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs::identical_authority_recomputes_identical_plan`; `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs::test_modes_match_accepts`; `crates/z00z_simulator/tests/test_scenario_11.rs::scenario11_happy_path_consistent` | `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs::planner_config_drift_rejects`; `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs::stale_route_digest_rejects`; `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs::mixed_planner_generation_rejects`; `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs::copied_primary_plan_bytes_reject`; `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs::test_rejects_dispatch_drift` | `live` | `scenario_11/quorum/route_plan_report.json`; `scenario_11/quorum/report_honesty.json`; `crates/z00z_runtime/aggregators/README.md` | `067-12` |
| `planner HA` | `crates/z00z_runtime/aggregators` | `scenario_11/quorum/report_honesty.json`, `RoutePlanReport.planner_ha_claim_level`, `ReportHonesty.claim_levels` | `crates/z00z_simulator/tests/test_scenario_11.rs::scenario11_report_honesty_rejects_overclaims` | `crates/z00z_simulator/tests/test_scenario_11.rs::scenario11_report_honesty_rejects_overclaims` | `live-claim-removed` | `scenario_11/quorum/route_plan_report.json`; `scenario_11/quorum/report_honesty.json`; `.planning/phases/067-Sharded-Concensus/067-verdict.md` | `067-12` |
