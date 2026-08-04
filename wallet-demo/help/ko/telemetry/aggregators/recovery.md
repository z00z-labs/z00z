---
id: telemetry.aggregators.recovery
title: 애그리게이터 복구
summary: 커밋된 route, generation, primary와 journal lineage에 대한 재시작 및 secondary takeover 검사를 설명합니다.
scope: context
---
## 이 화면 사용 {#current-view}
- `ShardRecoveryRecord`, recovery intent, durable state와 execution ticket을 확인하세요.
- 사용 불가는 커밋된 복구 스냅샷이 연결되지 않았다는 뜻입니다.

## Fail-closed 경계
- Generation, primary, shard, batch, route 또는 lineage가 틀리면 거부됩니다.
- Renderer는 failover를 시작하거나 storage recovery truth를 변경할 수 없습니다.
