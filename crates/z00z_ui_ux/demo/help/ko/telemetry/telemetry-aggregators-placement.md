---
id: telemetry.aggregators.placement
title: 애그리게이터 배치
summary: 런타임 소유의 shard generation, primary, secondary 준비 상태와 journal lineage를 설명합니다.
scope: context
---
## 이 화면 사용 {#current-view}
- 글로벌 토폴로지를 추정하지 말고 `ShardPlacementView` 계약을 확인하세요.
- 사용 불가는 현재 placement table 관측이 연결되지 않았다는 뜻입니다.

## Fail-closed 경계
- Placement table은 정확한 shard와 routing generation을 소유해야 합니다.
- Aggregator ID는 운영 데이터이며 endpoint와 지갑 identity는 숨겨집니다.
