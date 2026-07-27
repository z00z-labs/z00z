---
id: telemetry.aggregators.ingress
title: 애그리게이터 인그레스
summary: 런타임이 트랜잭션 또는 클레임 payload를 digest에 바인딩된 작업 항목으로 승인하는 방식을 설명합니다.
scope: context
---
## 이 화면 사용 {#current-view}
- `WorkPayload`에서 `WorkItem` 또는 `RejectRecord`로 이어지는 계약을 확인하세요.
- 사용 불가는 최신 승인 스냅샷이 없다는 뜻이며 승인이나 거부를 뜻하지 않습니다.

## Fail-closed 경계
- Object package 바인딩은 admission digest와 intake identity를 변경합니다.
- Raw payload, 수신자, 메모와 지갑 로컬 경로는 도움말에 포함되지 않습니다.
