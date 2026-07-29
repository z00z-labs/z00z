---
id: wallet.merge
title: "Cartera: Combinar"
route: wallet.merge-split
scope: context
---

# Cartera: Combinar

[TOC]

## Vista de la aplicación {#current-view}

![Vista de combinación de la cartera](help/assets/en/wallet-merge.png)

Esta imagen se captura desde la vista Combinar actual de la Demo.

## Descripción general {#overview}

Combinar reúne dos o más fragmentos confidenciales compatibles de un activo en una sola salida. La salida conserva los mismos `definition_id` y `serial_id` base, y su importe equivale a la suma de las entradas seleccionadas. La operación cambia la organización de las salidas, pero no modifica la definición del activo ni crea oferta.

Los candidatos se agrupan tanto por definición como por serie. Los fragmentos de grupos distintos no se pueden combinar, aunque utilicen el mismo símbolo visible.

## Cómo usar esta vista {#how-to-use-this-view}

1. Confirme la cartera activa y la red en el encabezado de la aplicación.
2. Seleccione **Combinar**.
3. Elija al menos dos fragmentos disponibles de un mismo grupo compatible.
4. Compruebe el número de entradas, el importe total de salida, la definición y la serie.
5. Seleccione **Vista previa de la combinación** y revise cada entrada y la única salida propuesta.
6. Continúe únicamente en una cartera nativa que pueda volver a comprobar la autorización, las comisiones, el envío y la conciliación.

## Términos y controles {#terms-and-controls}

| Término o control | Explicación |
| --- | --- |
| ID de definición | Identificador inmutable del tipo de activo y su política. Todas las entradas seleccionadas deben compartirlo. |
| ID de serie | Serie de emisión base. Todas las entradas y la salida combinada conservan la misma serie. |
| ID de activo | Identificador de una salida confidencial concreta. Los fragmentos compatibles pueden tener ID de activo distintos. |
| Grupo compatible | Fragmentos disponibles con el mismo ID de definición y de serie. |
| Bloqueado | El fragmento se muestra como contexto, pero no se puede seleccionar. |
| Salida total | Suma exacta de las entradas seleccionadas antes de aplicar cualquier política de comisión nativa independiente. |
| Vista previa de la combinación | Intención solo para revisión que muestra las entradas y la salida propuesta; no firma ni envía nada. |

## Seguridad y límites {#safety-and-limits}

- Esta interfaz nunca combina definiciones ni series base distintas.
- La cartera nativa debe rechazar entradas bloqueadas, gastadas, congeladas, quemadas, penalizadas o no disponibles por cualquier otro motivo, aunque una pantalla obsoleta las mostrara antes.
- Combinar fragmentos puede facilitar la correlación de entradas relacionadas. Revise el impacto sobre la privacidad antes de realizar operaciones repetidas o con patrones marcados.
- La Demo de JavaScript utiliza datos públicos y se detiene en la vista previa. No guarda claves, demuestra propiedad, crea firmas, cobra comisiones, envía paquetes ni concilia resultados inciertos.
- El helper actual `wallet.asset.merge_assets` es una superficie de compatibilidad y no reclama autoridad canónica para conciliar el registro. La integración nativa debe dirigir la confirmación por la ruta de transacción autorizada de la cartera.

<!-- help-sync:source {"page_path":"wallet/merge-split/index.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-merge.png","topic_id":"wallet.merge"} -->
