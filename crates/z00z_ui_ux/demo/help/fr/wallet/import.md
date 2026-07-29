---
id: wallet.import
title: Importer
summary: Importer examine un paquet d’actif public depuis le disque avant de le remettre au portefeuille actif.
scope: context
---
## Utiliser cette vue {#current-view}
- Choisissez un paquet JSON `AssetPkgWire` de 64 Kio maximum.
- Vérifiez le portefeuille, le réseau, la classe, le montant, le numéro de série, le domaine, les états et la liaison du propriétaire.
- Sélectionnez **Importer l’actif** ; le portefeuille natif vérifie la cryptographie, la propriété, la répétition et le claim.

## Fonctionnement local et sûr
- Le champ `secret` est interdit et le chemin absolu du fichier n’est ni conservé ni envoyé au RPC.
- Le résultat distingue un nouvel import, un actif existant et une cause explicite `IMPORT_*`.
