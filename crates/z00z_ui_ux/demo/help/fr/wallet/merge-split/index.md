---
id: wallet.merge
title: "Portefeuille : Fusionner"
route: wallet.merge-split
scope: context
---

# Portefeuille : Fusionner

[TOC]

## Vue de l’application {#current-view}

![Vue de fusion du portefeuille](help/assets/en/wallet-merge.png)

Cette image est capturée depuis la vue Fusionner actuelle de la Demo.

## Vue d’ensemble {#overview}

Fusionner regroupe au moins deux fragments d’actif confidentiels compatibles en une seule sortie. La sortie conserve les mêmes `definition_id` et `serial_id` de base, et son montant est égal à la somme des entrées sélectionnées. L’opération modifie l’organisation des sorties sans modifier la définition de l’actif ni créer d’offre.

Les candidats sont regroupés à la fois par définition et par série. Des fragments appartenant à des groupes différents ne peuvent pas être fusionnés, même s’ils affichent le même symbole.

## Utiliser cet écran {#how-to-use-this-view}

1. Vérifiez le portefeuille actif et le réseau dans l’en-tête de l’application.
2. Sélectionnez **Fusionner**.
3. Choisissez au moins deux fragments disponibles dans un même groupe de compatibilité.
4. Vérifiez le nombre d’entrées, le montant total de la sortie, la définition et la série.
5. Sélectionnez **Aperçu de la fusion** et vérifiez chaque entrée ainsi que la sortie unique proposée.
6. Ne continuez que dans un portefeuille natif capable de revérifier l’autorisation, les frais, l’envoi et le rapprochement.

## Termes et commandes {#terms-and-controls}

| Terme ou commande | Explication |
| --- | --- |
| ID de définition | Identifiant immuable du type d’actif et de sa politique. Toutes les entrées sélectionnées doivent le partager. |
| ID de série | Série d’émission de base. Toutes les entrées et la sortie fusionnée conservent la même série. |
| ID d’actif | Identifiant d’une sortie confidentielle précise. Des fragments compatibles peuvent avoir des ID d’actif différents. |
| Groupe compatible | Fragments disponibles ayant les mêmes ID de définition et de série. |
| Verrouillé | Le fragment reste visible pour le contexte, mais ne peut pas être sélectionné. |
| Sortie totale | Somme exacte des entrées sélectionnées avant toute politique de frais native distincte. |
| Aperçu de la fusion | Intention de vérification montrant les entrées et la sortie proposée ; elle ne signe ni n’envoie rien. |

## Sécurité et limites {#safety-and-limits}

- Cette interface ne fusionne jamais des définitions ou des séries de base différentes.
- Le portefeuille natif doit rejeter toute entrée verrouillée, dépensée, gelée, brûlée, pénalisée ou autrement indisponible, même si un écran obsolète l’affichait auparavant.
- La fusion de fragments peut faciliter la corrélation d’entrées associées. Évaluez l’impact sur la confidentialité avant des opérations répétées ou très régulières.
- La Demo JavaScript utilise des données publiques et s’arrête à l’aperçu. Elle ne détient pas de clés, ne prouve pas la propriété, ne crée pas de signatures, ne facture pas de frais, n’envoie pas de paquet et ne rapproche pas un résultat incertain.
- L’assistant actuel `wallet.asset.merge_assets` est une surface de compatibilité et ne revendique pas l’autorité canonique de rapprochement du registre. L’intégration native doit faire confirmer l’opération par le chemin de transaction faisant autorité dans le portefeuille.

<!-- help-sync:source {"page_path":"wallet/merge-split/index.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-merge.png","topic_id":"wallet.merge"} -->
