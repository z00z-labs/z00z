---
id: telemetry.aggregators.ingress
title: Entrada do agregador
summary: Esta vista explica como o runtime admite uma transação ou claim como trabalho ligado a um digest.
scope: context
---
## Utilizar esta vista {#current-view}
- Verifique o contrato `WorkPayload` para `WorkItem` ou `RejectRecord`.
- Indisponível significa que não há snapshot recente de admissão, não que foi aceite ou rejeitado.

## Limite fail-closed
- Ligar um object package altera o digest de admissão e a identidade de entrada.
- Payloads brutos, destinatários, notas e rotas locais da carteira não entram na Ajuda.
