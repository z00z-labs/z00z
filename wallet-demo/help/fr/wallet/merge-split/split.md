---
id: wallet.split
title: "Portefeuille : Diviser"
route: wallet.merge-split
scope: context
---

# Portefeuille : Diviser

[TOC]

## Vue de l’application {#current-view}

![Vue de division du portefeuille](help/assets/en/wallet-split.png)

Cette image est capturée depuis la vue Diviser actuelle de la Demo.

## Vue d’ensemble {#overview}

Diviser consomme un fragment d’actif confidentiel et prépare au moins deux sorties. Chaque sortie conserve les `definition_id` et `serial_id` de base de la source, et la somme de tous les montants positifs doit être exactement égale au montant d’entrée. L’opération modifie l’organisation des sorties sans modifier la définition de l’actif ni créer d’offre.

Chaque fragment obtenu reçoit sa propre identité de sortie tout en restant dans la même série d’émission.

## Utiliser cet écran {#how-to-use-this-view}

1. Vérifiez le portefeuille actif et le réseau dans l’en-tête de l’application.
2. Sélectionnez **Diviser**.
3. Choisissez un fragment source disponible.
4. Saisissez entre deux et huit montants de sortie positifs.
5. Vérifiez que **Conservation** indique **Exacte**.
6. Sélectionnez **Aperçu de la division** et vérifiez la source et chaque sortie proposée.
7. Ne continuez que dans un portefeuille natif capable de revérifier l’autorisation, les frais, l’envoi et le rapprochement.

## Termes et commandes {#terms-and-controls}

| Terme ou commande | Explication |
| --- | --- |
| Actif source | Fragment disponible unique consommé par la division proposée. |
| ID de définition | Identifiant immuable du type d’actif et de sa politique. Chaque sortie conserve la définition source. |
| ID de série | Série d’émission de base. Chaque sortie conserve la série de la source. |
| Répartition des sorties | Entre deux et huit montants positifs attribués aux sorties proposées. |
| Conservation | Égalité exacte entre le montant d’entrée et la somme de tous les montants de sortie. |
| Ajouter une sortie | Ajoute un autre champ de montant positif, dans la limite de l’interface. |
| Aperçu de la division | Intention de vérification montrant la source et les sorties proposées ; elle ne signe ni n’envoie rien. |

## Sécurité et limites {#safety-and-limits}

- La division ne modifie jamais la définition source ni la série de base.
- Les répartitions nulles, négatives, excessives ou qui ne conservent pas le montant doivent être rejetées.
- Le portefeuille natif doit rejeter une source devenue verrouillée, dépensée, gelée, brûlée, pénalisée ou autrement indisponible.
- Des répartitions répétées ou inhabituellement régulières peuvent faciliter la corrélation des sorties associées.
- La Demo JavaScript utilise des données publiques et s’arrête à l’aperçu. Elle ne détient pas de clés, ne prouve pas la propriété, ne crée pas de signatures, ne facture pas de frais, n’envoie pas de paquet et ne rapproche pas un résultat incertain.
- L’assistant actuel `wallet.asset.split_asset` est une surface de compatibilité et ne revendique pas l’autorité canonique de rapprochement du registre. L’intégration native doit faire confirmer l’opération par le chemin de transaction faisant autorité dans le portefeuille.

<!-- help-sync:source {"page_path":"wallet/merge-split/split.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-split.png","topic_id":"wallet.split"} -->
