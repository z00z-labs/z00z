---
id: telemetry.aggregators.publication
title: 애그리게이터 게시
summary: 정렬된 batch가 checkpoint, quorum, DA 및 lifecycle evidence에 바인딩되는 방식을 설명합니다.
scope: context
---
## 이 화면 사용 {#current-view}
- `PublicationRequest`에서 `PublishedBatch`와 `PublicationRecord`로 이어지는 흐름을 확인하세요.
- 사용 불가는 검증된 게시 또는 readiness bundle이 연결되지 않았다는 뜻입니다.

## Fail-closed 경계
- Provider, height, manifest, payload, statement 또는 evidence가 불완전하거나 다르면 거부됩니다.
- Storage가 checkpoint root, proof와 lifecycle truth를 소유합니다.
