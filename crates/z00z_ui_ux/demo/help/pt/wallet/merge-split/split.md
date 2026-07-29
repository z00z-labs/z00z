---
id: wallet.split
title: "Carteira: Dividir"
route: wallet.merge-split
scope: context
---

# Carteira: Dividir

[TOC]

## Vista da aplicação {#current-view}

![Vista de divisão da carteira](help/assets/en/wallet-split.png)

Esta imagem é capturada a partir da vista Dividir atual da Demo.

## Visão geral {#overview}

Dividir consome um fragmento confidencial de um ativo e prepara duas ou mais saídas. Cada saída mantém os `definition_id` e `serial_id` base da origem, e todos os montantes positivos devem somar exatamente o montante de entrada. A operação altera a organização das saídas, mas não altera a definição do ativo nem cria oferta.

Cada fragmento resultante recebe a sua própria identidade de saída concreta e continua a fazer parte da mesma série de emissão.

## Como utilizar esta vista {#how-to-use-this-view}

1. Confirme a carteira ativa e a rede no cabeçalho da aplicação.
2. Selecione **Dividir**.
3. Escolha um fragmento de origem disponível.
4. Introduza entre dois e oito montantes de saída positivos.
5. Confirme que **Conservação** apresenta **Exata**.
6. Selecione **Pré-visualizar divisão** e reveja a origem e cada saída proposta.
7. Continue apenas numa carteira nativa que possa voltar a verificar autorização, taxas, envio e reconciliação.

## Termos e controlos {#terms-and-controls}

| Termo ou controlo | Explicação |
| --- | --- |
| Ativo de origem | Único fragmento disponível consumido pela divisão proposta. |
| ID da definição | Identificador imutável do tipo de ativo e da respetiva política. Cada saída mantém a definição da origem. |
| ID da série | Série de emissão base. Cada saída mantém a série da origem. |
| Distribuição de saídas | Entre dois e oito montantes positivos atribuídos às saídas propostas. |
| Conservação | Igualdade exata entre o montante de entrada e a soma de todos os montantes de saída. |
| Adicionar saída | Adiciona outro campo de montante positivo até ao limite da interface. |
| Pré-visualizar divisão | Intenção apenas para revisão que mostra a origem e as saídas propostas; não assina nem envia. |

## Segurança e limites {#safety-and-limits}

- Dividir nunca altera a definição da origem nem a série base.
- Distribuições nulas, negativas, excessivas ou que não conservem o montante devem ser rejeitadas.
- A carteira nativa deve rejeitar uma origem que tenha ficado bloqueada, gasta, congelada, queimada, penalizada ou indisponível por outro motivo.
- Distribuições repetidas ou com padrões invulgares podem facilitar a correlação de saídas relacionadas.
- A Demo JavaScript utiliza dados públicos e termina na pré-visualização. Não guarda chaves, prova propriedade, cria assinaturas, cobra taxas, envia pacotes nem reconcilia resultados incertos.
- O helper atual `wallet.asset.split_asset` é uma superfície de compatibilidade e não reivindica autoridade canónica de reconciliação do registo. A integração nativa deve encaminhar a confirmação pelo percurso de transação autorizado da carteira.

<!-- help-sync:source {"page_path":"wallet/merge-split/split.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-split.png","topic_id":"wallet.split"} -->
