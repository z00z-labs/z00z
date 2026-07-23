"use strict";

window.Z00ZI18n.registerLocale("es", {
  common: { unavailable: "No disponible", readOnly: "Solo lectura", chain: "Cadena", chainLocked: "Elige con cuidado. La cadena no se puede cambiar después de crear la cartera.", on: "Activado", off: "Desactivado", close: "Cerrar" },
  app: {
    documentTitle: "Z00Z Wallet — Concepto interactivo", wallets: "Carteras", network: "Red",
    addWallet: "Añadir cartera", removeWallet: "Eliminar cartera", settings: "Ajustes", home: "Inicio",
    homeContext: "Tu dinero privado de un vistazo", settingsContext: "Preferencias de la aplicación", walletContext: "Cartera {wallet}", interactiveConcept: "Concepto interactivo · sin fondos reales", conceptBuild: "Concepto 0.4 · sin fondos reales", logOut: "Cerrar sesión",
    general: "General", language: "Idioma", languageHelp: "Idioma utilizado en toda esta aplicación de cartera",
    notifications: "Notificaciones", notificationsHelp: "Mostrar actualizaciones de la cartera y acciones necesarias",
    regionalFormat: "Formato regional", regionalFormatHelp: "Controla fechas, números y separadores decimales independientemente del idioma",
    timeZone: "Zona horaria", timeZoneHelp: "Las marcas de tiempo se guardan en UTC y se muestran en esta zona horaria",
    networkUnits: "Unidades de red", networkUnitsHelp: "Las tasas de red usan bits decimales por segundo",
    decimalBitrate: "Bits decimales por segundo", translationCoverage: "Cobertura de traducción",
    translationCoverageHelp: "{count} catálogos de idiomas están sincronizados con las claves fuente en inglés",
    languageChanged: "Idioma cambiado localmente."
  },
  nav: { home: "Inicio", assets: "Activos", history: "Historial", swap: "Intercambio", exchange: "Mercado", staking: "Staking", backup: "Copia de seguridad", settings: "Ajustes" },
  network: { routeTelemetry: "Telemetría de ruta", carrierTelemetry: "Telemetría de transporte", publicationTelemetry: "Telemetría de publicación" },
  walletShell: {
    balanceAvailable: "{value} disponible",
    current: "Actual", scanning: "Analizando", identityAria: "Cambiar de cartera. Cartera actual: {wallet}", lockLabel: "Cartera {wallet}", copyAddress: "Copiar la dirección completa de la cartera {wallet}",
    available: "Disponible", locked: "Bloqueado", pendingIn: "Entrada pendiente", pendingOut: "Salida pendiente", routeTelemetry: "Telemetría de ruta"
  },
  assets: {
    wallet: "Cartera", sections: "Secciones de la cartera", sectionAssets: "Activos", sectionAssetsHelp: "Valor disponible y en propiedad", sectionVouchers: "Vales", sectionVouchersHelp: "Valor condicional", sectionPermissions: "Permisos", sectionPermissionsHelp: "Autoridad limitada",
    family: "Familia de activos", title: "Tus activos", description: "El efectivo nativo, los tokens emitidos y los coleccionables permanecen diferenciados. Solo el efectivo nativo disponible para gastar entra en Disponible.", send: "Enviar", moneyTotals: "Totales monetarios", available: "Disponible", readyToUse: "Listo para usar", receiving: "Recibiendo", sending: "Enviando", waitingToSettle: "Pendiente de liquidación",
    filters: "Filtros de activos", all: "Todos", filterCoins: "Monedas", filterTokens: "Tokens", filterNfts: "NFT", kindCoin: "Moneda", kindToken: "Token", kindCollectible: "NFT", needsReview: "Requiere revisión", owned: "Activos propios", ownedHelp: "La clase, la confianza y la disponibilidad para gastar son explícitas", name: "Nombre", balance: "Saldo", value: "Valor", price: "Precio",
    viewDetails: "Ver detalles de {asset}", receiveAsset: "Recibir {asset}", sendAsset: "Enviar {asset}", receive: "Recibir", noMarketFeed: "Sin fuente de mercado", nativeCatalog: "Activo nativo · catálogo fiable", declaredDomain: "Dominio declarado · requiere revisión", uniqueCollectible: "Coleccionable único · metadatos disponibles", excludedNotice: "Los vales, permisos, objetos en cuarentena, tokens no nativos, coleccionables y activos de compatibilidad experimental se excluyen de Disponible.",
    noVouchers: "Todavía no hay vales", noVouchersHelp: "Crea un vale cuando este monedero necesite valor condicional transferible.", createVoucher: "Crear vale", noPermissions: "Todavía no hay permisos", noPermissionsHelp: "Crea un permiso limitado cuando este monedero necesite autoridad transferible.", createPermission: "Crear permiso"
  },
  history: {
    honestSettlement: "Liquidación transparente", title: "Todo lo que cambió", description: "El envío y la liquidación final se muestran como estados distintos. Abre un elemento para ver su recibo y cronología técnica.", filters: "Filtros del historial", all: "Todo", assets: "Activos", vouchers: "Vales", permissions: "Permisos", system: "Sistema", needsAttention: "Requiere atención", search: "Buscar en el historial", results: "Resultados del historial",
    settling: "En liquidación", settled: "Liquidado", active: "Activo", ready: "Listo", paymentTo: "Pago a {recipient}", sentWaiting: "Enviado · esperando liquidación", allocationClaimed: "Asignación reclamada", verifiedClaimWaiting: "Reclamación verificada · esperando liquidación", receivedFrom: "Recibido de {sender}", yesterday: "Ayer", travelRefundVoucher: "Vale de reembolso de viaje", offeredReviewBefore: "Ofrecido · revisar antes del {date}", deliveryReceiptAccess: "Acceso al recibo de entrega", dataAccessUsesRemain: "Acceso a datos · quedan {used} de {total} usos", uses: "{count} usos", jul12: "12 jul", localBackupCreated: "Copia local creada", integrityPassed: "Comprobación de integridad aprobada", jul10: "10 jul", transferFrom: "Transferencia desde {wallet}", jul3: "3 jul", recoveryCheckCompleted: "Comprobación de recuperación completada", localVerificationPassed: "Verificación local aprobada", jun30: "30 jun", jul21: "21 jul", waitingToSettle: "Esperando liquidación", minutesAgo: "hace {count} min", noMatching: "No hay historial coincidente", tryAnother: "Prueba otro filtro o término de búsqueda.", details: "Detalles del historial", status: "Estado", when: "Cuándo", fee: "Comisión", feeIncluded: "Incluida", feeNotApplicable: "No aplicable", privacy: "Privacidad", privacyValue: "Simulación de destino · sin telemetría en directo", carrierChain: "Transporte y cadena", carrierChainValue: "Destino Reticulum · maqueta de red principal", technicalDetails: "Detalles técnicos", idLabel: "ID", lifecycleLabel: "Ciclo de vida", lifecyclePending: "creado → enviado → admitido", lifecycleConfirmed: "creado → enviado → admitido → confirmado", receiptLabel: "Recibo", copyReceipt: "Copiar recibo", done: "Listo"
  },
  staking: {
    eyebrow: "Staking de cartera", heading: "Hacer staking desde {wallet}", description: "Las condiciones de staking y el valor pendiente permanecen separados del saldo Disponible para gastar de la cartera.", badge: "Red principal · concepto", totals: "Totales de staking", availableToStake: "Disponible para staking", walletValueBefore: "Valor de la cartera antes de enviar el staking", staked: "En staking", nothingDelegated: "No se delegó nada en este concepto", rewards: "Recompensas", accrualNotSimulated: "No se simula la acumulación", prepare: "Preparar staking", prepareHelp: "Elige el importe y revisa las condiciones del validador antes de autorizar.", amount: "Importe", availableBalance: "Disponible: {value}", validator: "Validador", validatorPlaceholder: "Elige tras verificar la cadena", review: "Revisar staking", safeguards: "Salvaguardas de staking", validatorStatus: "Estado del validador", unlockPeriod: "Periodo de desbloqueo", notSelected: "No seleccionado", notProjected: "No proyectado", notice: "El concepto nunca estima el rendimiento ni oculta el riesgo de bloqueo. Un staking queda pendiente hasta la liquidación en cadena."
  },
  settings: { sections: "Secciones de ajustes", application: "Aplicación", connectivity: "Conectividad", general: "General", generalHelp: "Idioma y notificaciones de la aplicación", appearance: "Apariencia", appearanceHelp: "Tema, paleta, densidad y resaltado YAML", networkPrivacy: "Red", networkPrivacyHelp: "Ruta privada, cadena y sincronización", networkSections: "Secciones de red", overview: "Resumen" },
  walletSettings: { password: "Contraseña de la cartera", passwordHelp: "Cambia localmente la contraseña de esta cartera. Se requiere la contraseña actual antes de aceptar una nueva.", changePassword: "Cambiar contraseña", changePasswordTitle: "Cambiar contraseña de la cartera", changePasswordSubtitle: "Verifica la contraseña actual y elige una nueva", currentPassword: "Contraseña actual", newPassword: "Nueva contraseña", confirmNewPassword: "Confirmar nueva contraseña", passwordChangeHint: "Usa al menos 8 caracteres. Las contraseñas se borran inmediatamente de la demo.", changePasswordSubmit: "Cambiar contraseña", passwordChangedTitle: "Contraseña actualizada", passwordChangedResult: "La contraseña solo se cambió en este concepto local. La demo borra cada entrada y no conserva ningún secreto.", passwordCurrentError: "Introduce la contraseña actual (al menos 8 caracteres).", passwordNewError: "Usa al menos 8 caracteres para la nueva contraseña.", passwordSameError: "Elige una contraseña distinta de la actual.", passwordMismatchError: "Las nuevas contraseñas no coinciden." },
  reticulum: {
    title: "Telemetría de Reticulum",
    summary: "Una vista local de evidencia de transporte y entrega. Ayuda a decidir la disponibilidad sin convertir metadatos de transporte en analítica de usuario.",
    localCapability: "Capacidad local no disponible",
    localCapabilityHelp: "Esta demo de cartera no tiene un puente local de estado Reticulum registrado. No se inventan datos de transporte que parezcan reales.",
    tabs: { overview: "Resumen", node: "Nodo", interfaces: "Interfaces", radio: "Radio", entrypoints: "Puntos de entrada", paths: "Rutas", probes: "Sondas", links: "Enlaces" }
  },
  aggregators: {
    title: "Telemetría de agregadores",
    summary: "Evidencia de servicios y publicaciones para la agregación, solo de lectura. Este espacio nunca recibe claves, frases semilla ni secretos de política de la cartera.",
    localCapability: "Capacidad local no disponible",
    localCapabilityHelp: "La cartera no tiene un puente registrado al estado de los agregadores, por lo que esta página muestra correctamente la falta de disponibilidad.",
    tabs: { overview: "Resumen" }
  },
  onionnet: {
    title: "Telemetría de OnionNet",
    summary: "Una vista consciente de los límites de datos del estado determinista del plano de control, la evidencia local y la salud sintética agregada. Este espacio nunca cambia una ruta ni una política de privacidad.",
    localCapability: "Capacidad local no disponible",
    localCapabilityHelp: "Esta demo de cartera no tiene un puente de estado OnionNet registrado. No se inventan valores de ruta, topología o privacidad que parezcan reales.",
    tabs: { overview: "Resumen", epoch: "Época", privacy: "Umbral de privacidad", transport: "Transporte", queues: "Colas y repeticiones", probation: "Período de prueba", ingress: "Límite de entrada" }
  },
  status: { up: "Activo", down: "Inactivo", connecting: "Conectando", degraded: "Degradado" },
  units: { bitPerSecond: "{value} bit/s", kilobitPerSecond: "{value} kbit/s", megabitPerSecond: "{value} Mbit/s" }
});
