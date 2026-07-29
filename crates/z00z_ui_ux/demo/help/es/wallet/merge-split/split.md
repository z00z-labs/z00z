---
id: wallet.split
title: "Cartera: Dividir"
route: wallet.merge-split
scope: context
---

# Cartera: Dividir

[TOC]

## Vista de la aplicación {#current-view}

![Vista de división de la cartera](help/assets/en/wallet-split.png)

Esta imagen se captura desde la vista Dividir actual de la Demo.

## Descripción general {#overview}

Dividir consume un fragmento confidencial de un activo y prepara dos o más salidas. Cada salida conserva los `definition_id` y `serial_id` base del origen, y todos los importes positivos deben sumar exactamente el importe de entrada. La operación cambia la organización de las salidas, pero no modifica la definición del activo ni crea oferta.

Cada fragmento resultante recibe su propia identidad de salida concreta y sigue formando parte de la misma serie de emisión.

## Cómo usar esta vista {#how-to-use-this-view}

1. Confirme la cartera activa y la red en el encabezado de la aplicación.
2. Seleccione **Dividir**.
3. Elija un fragmento de origen disponible.
4. Introduzca entre dos y ocho importes de salida positivos.
5. Confirme que **Conservación** indica **Exacta**.
6. Seleccione **Vista previa de la división** y revise el origen y cada salida propuesta.
7. Continúe únicamente en una cartera nativa que pueda volver a comprobar la autorización, las comisiones, el envío y la conciliación.

## Términos y controles {#terms-and-controls}

| Término o control | Explicación |
| --- | --- |
| Activo de origen | Único fragmento disponible que consume la división propuesta. |
| ID de definición | Identificador inmutable del tipo de activo y su política. Cada salida conserva la definición de origen. |
| ID de serie | Serie de emisión base. Cada salida conserva la serie del origen. |
| Asignación de salidas | Entre dos y ocho importes positivos asignados a las salidas propuestas. |
| Conservación | Igualdad exacta entre el importe de entrada y la suma de todos los importes de salida. |
| Añadir salida | Añade otro campo de importe positivo hasta el límite de la interfaz. |
| Vista previa de la división | Intención solo para revisión que muestra el origen y las salidas propuestas; no firma ni envía nada. |

## Seguridad y límites {#safety-and-limits}

- Dividir nunca cambia la definición de origen ni la serie base.
- Se deben rechazar asignaciones nulas, negativas, excesivas o que no conserven el importe.
- La cartera nativa debe rechazar un origen que haya quedado bloqueado, gastado, congelado, quemado, penalizado o no disponible por cualquier otro motivo.
- Las asignaciones repetidas o con patrones inusuales pueden facilitar la correlación de salidas relacionadas.
- La Demo de JavaScript utiliza datos públicos y se detiene en la vista previa. No guarda claves, demuestra propiedad, crea firmas, cobra comisiones, envía paquetes ni concilia resultados inciertos.
- El helper actual `wallet.asset.split_asset` es una superficie de compatibilidad y no reclama autoridad canónica para conciliar el registro. La integración nativa debe dirigir la confirmación por la ruta de transacción autorizada de la cartera.

<!-- help-sync:source {"page_path":"wallet/merge-split/split.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-split.png","topic_id":"wallet.split"} -->
