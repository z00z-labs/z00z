---
id: wallet.import
title: Importar
summary: Importar revisa un paquete público de activo del disco antes de entregarlo a la cartera activa.
scope: context
---
## Usar esta vista {#current-view}
- Elija un paquete JSON `AssetPkgWire` de hasta 64 KiB.
- Revise cartera, red, clase, importe, ID de serie, dominio, estados y vínculo de propietario.
- Seleccione **Importar activo**; la cartera nativa verifica criptografía, propiedad, repetición y conflictos del claim.

## Funcionamiento local y seguro
- El campo `secret` está prohibido y la ruta absoluta no se guarda ni se envía al RPC.
- El resultado distingue una importación nueva, un activo existente y un motivo `IMPORT_*` explícito.
