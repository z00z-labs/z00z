---
id: telemetry.aggregators.planning
title: 애그리게이터 계획
summary: 결제 권한을 주장하지 않고 결정론적 batch와 shard route 바인딩을 설명합니다.
scope: context
---
## 이 화면 사용 {#current-view}
- Planner mode, route generation, intake와 operation 수, digest 소유권을 확인하세요.
- 사용 불가는 검증된 `BatchPlanned` 스냅샷이 연결되지 않았다는 뜻입니다.

## Fail-closed 경계
- 설정, generation, route-table digest와 재계산된 plan은 일치해야 합니다.
- 계획은 settlement, publication 또는 storage truth를 확정하지 않습니다.
