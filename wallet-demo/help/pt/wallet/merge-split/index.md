---
id: wallet.merge
title: "Carteira: Combinar"
route: wallet.merge-split
scope: context
---

# Carteira: Combinar

[TOC]

## Vista da aplicação {#current-view}

![Vista de combinação da carteira](help/assets/en/wallet-merge.png)

Esta imagem é capturada a partir da vista Combinar atual da Demo.

## Visão geral {#overview}

Combinar reúne dois ou mais fragmentos confidenciais compatíveis de um ativo numa única saída. A saída mantém os mesmos `definition_id` e `serial_id` base, e o seu montante é igual à soma das entradas selecionadas. A operação altera a organização das saídas, mas não altera a definição do ativo nem cria oferta.

Os candidatos são agrupados tanto por definição como por série. Fragmentos de grupos diferentes não podem ser combinados, mesmo quando utilizam o mesmo símbolo visível.

## Como utilizar esta vista {#how-to-use-this-view}

1. Confirme a carteira ativa e a rede no cabeçalho da aplicação.
2. Selecione **Combinar**.
3. Escolha pelo menos dois fragmentos disponíveis do mesmo grupo compatível.
4. Verifique o número de entradas, o montante total da saída, a definição e a série.
5. Selecione **Pré-visualizar combinação** e reveja cada entrada e a única saída proposta.
6. Continue apenas numa carteira nativa que possa voltar a verificar autorização, taxas, envio e reconciliação.

## Termos e controlos {#terms-and-controls}

| Termo ou controlo | Explicação |
| --- | --- |
| ID da definição | Identificador imutável do tipo de ativo e da respetiva política. Todas as entradas selecionadas devem partilhá-lo. |
| ID da série | Série de emissão base. Todas as entradas e a saída combinada mantêm a mesma série. |
| ID do ativo | Identificador de uma saída confidencial concreta. Fragmentos compatíveis podem ter ID de ativo diferentes. |
| Grupo compatível | Fragmentos disponíveis com os mesmos ID da definição e da série. |
| Bloqueado | O fragmento permanece visível para contexto, mas não pode ser selecionado. |
| Saída total | Soma exata das entradas selecionadas antes de qualquer política de taxa nativa separada. |
| Pré-visualizar combinação | Intenção apenas para revisão que mostra as entradas e a saída proposta; não assina nem envia. |

## Segurança e limites {#safety-and-limits}

- Esta interface nunca combina definições ou séries base diferentes.
- A carteira nativa deve rejeitar entradas bloqueadas, gastas, congeladas, queimadas, penalizadas ou indisponíveis por outro motivo, mesmo que uma vista desatualizada as tenha mostrado.
- Combinar fragmentos pode facilitar a correlação de entradas relacionadas. Reveja o impacto na privacidade antes de operações repetidas ou com padrões marcados.
- A Demo JavaScript utiliza dados públicos e termina na pré-visualização. Não guarda chaves, prova propriedade, cria assinaturas, cobra taxas, envia pacotes nem reconcilia resultados incertos.
- O helper atual `wallet.asset.merge_assets` é uma superfície de compatibilidade e não reivindica autoridade canónica de reconciliação do registo. A integração nativa deve encaminhar a confirmação pelo percurso de transação autorizado da carteira.

<!-- help-sync:source {"page_path":"wallet/merge-split/index.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-merge.png","topic_id":"wallet.merge"} -->
