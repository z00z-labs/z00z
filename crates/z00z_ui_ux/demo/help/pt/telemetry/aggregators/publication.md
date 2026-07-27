---
id: telemetry.aggregators.publication
title: Publicação do agregador
summary: Esta vista explica como um batch ordenado é ligado a checkpoint, quorum, DA e evidência de ciclo de vida.
scope: context
---
## Utilizar esta vista {#current-view}
- Siga `PublicationRequest` para `PublishedBatch` e `PublicationRecord`.
- Indisponível significa que não há publicação ou pacote de readiness verificado.

## Limite fail-closed
- Dados incompletos ou divergentes de provider, altura, manifesto, payload, statement ou evidence são rejeitados.
- Storage possui as raízes, provas e verdade de ciclo de vida do checkpoint.
