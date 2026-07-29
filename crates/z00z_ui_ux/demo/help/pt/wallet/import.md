---
id: wallet.import
title: Importar
summary: Importar analisa um pacote público de ativo do disco antes de o entregar à carteira ativa.
scope: context
---
## Utilizar esta vista {#current-view}
- Escolha um pacote JSON `AssetPkgWire` com até 64 KiB.
- Confirme carteira, rede, classe, montante, ID de série, domínio, estados e vínculo do proprietário.
- Selecione **Importar ativo**; a carteira nativa valida criptografia, propriedade, repetição e conflitos do claim.

## Funcionamento local e seguro
- O campo `secret` é proibido e o caminho absoluto não é guardado nem enviado ao RPC.
- O resultado distingue importação nova, ativo existente e uma razão `IMPORT_*` explícita.
