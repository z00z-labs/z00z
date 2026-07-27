---
id: telemetry.aggregators.ingress
title: Вход агрегатора
summary: Экран объясняет, как runtime принимает транзакцию или claim как рабочий элемент, связанный с digest.
scope: context
---
## Как использовать экран {#current-view}
- Проверяйте контракт `WorkPayload` → `WorkItem` или `RejectRecord`.
- Недоступно означает отсутствие свежего снимка admission, а не принятие или отказ.

## Fail-closed граница
- Привязка object package изменяет admission digest и intake identity.
- Raw payload, получатели, memo и локальные маршруты кошелька не попадают в Help.
