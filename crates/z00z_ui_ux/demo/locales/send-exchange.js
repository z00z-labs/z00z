"use strict";

(() => {
  const i18n = window.Z00ZI18n;
  if (!i18n?.extendLocale) throw new Error("Z00Z i18n extensions must load after the locale catalogues.");

  const english = {
    send: {
      familyNavigation: "Send object family",
      objectType: "Object type",
      spendableBalance: "Spendable balance",
      unit: "Unit",
      conditionalValue: "Conditional value",
      lifecycle: "Lifecycle",
      expires: "Expires",
      receiverAcceptance: "Receiver acceptance",
      required: "Required",
      authorityValue: "Authority value",
      zeroValue: "Zero-value",
      action: "Action",
      scope: "Scope",
      remainingUses: "Remaining uses",
      delegation: "Delegation",
      title: "Send privately",
      subtitle: "Recipient, object, and authorization",
      amount: "Amount",
      available: "Available: {value}",
      transferObject: "Transfer object",
      recipient: "Recipient or private request",
      recipientPlaceholder: "Paste or scan a private request",
      itemLabel: { asset: "Asset", voucher: "Voucher", permission: "Permission" },
      note: "Private note",
      optional: "optional",
      notePlaceholder: "What is this for?",
      cancel: "Cancel",
      review: "Review send",
      reviewTitle: "Review send",
      reviewSubtitle: "Check before authorizing",
      family: "Object family",
      familyName: { asset: "Asset", voucher: "Voucher", permission: "Permission" },
      item: "Object",
      recipientShort: "Recipient",
      from: "From",
      fee: "Fee",
      feeAtAuthorization: "Shown at authorization",
      notApplicable: "Not applicable",
      noteShort: "Note",
      confirmation: {
        asset: "This authorizes one asset transfer. Final settlement remains pending until confirmed.",
        voucher: "This transfers one conditional-value voucher subject to its policy and receiver acceptance.",
        permission: "This delegates bounded zero-value authority; it does not transfer money."
      },
      submit: { asset: "Send asset", voucher: "Send voucher", permission: "Send permission" },
      sent: { asset: "Asset sent", voucher: "Voucher sent", permission: "Permission sent" },
      settlementPending: "Waiting for final settlement",
      settling: "Sent · settling",
      value: "Value",
      nextUpdate: "Next update",
      automatic: "Automatic",
      viewHistory: "View history",
      empty: {
        asset: "No transferable assets",
        voucher: "No transferable vouchers",
        permission: "No transferable permissions"
      }
    },
    exchange: {
      executionModel: "Execution model",
      providerHyperliquid: "Hyperliquid Spot",
      providerNearIntents: "NEAR Intents",
      executionOrderBook: "Spot order book",
      executionSolver: "Cross-chain solver intent",
      title: "Build an exchange request",
      sourceAsset: "Source asset",
      amount: "Amount",
      destinationAsset: "Destination asset",
      orderType: "Order type",
      market: "Market",
      limit: "Limit",
      limitPrice: "Limit price",
      limitPricePlaceholder: "Enter a positive price",
      recipient: "Recipient",
      recipientPlaceholder: "Destination-chain address",
      refundAddress: "Refund address",
      refundPlaceholder: "Origin-chain refund address",
      slippage: "Slippage limit",
      deadline: "Quote deadline",
      review: "Review request",
      reviewTitle: "Review exchange request",
      pair: "Spot pair",
      route: "Route",
      mode: "Mode",
      exactInput: "Exact input",
      from: "From",
      to: "To",
      rate: "Rate",
      expectedOutput: "Expected output",
      minimumReceived: "Minimum received",
      fee: "Fees",
      eta: "Estimated time",
      depositAddress: "Deposit address",
      executionStatus: "Execution status",
      marketState: "Market state",
      connectorBoundary: "No provider connector is registered. Rate, fees, output, deposit address, and execution status remain unavailable.",
      editRequest: "Edit request",
      amountError: "Enter an amount within the available {balance} {unit}.",
      limitPriceError: "Enter a positive limit price.",
      addressError: "Enter both a destination recipient and an origin-chain refund address."
    }
  };

  const translations = {
    ru: {
      send: {
        familyNavigation: "Семейство отправляемого объекта", objectType: "Тип объекта", spendableBalance: "Доступный баланс", unit: "Единица", conditionalValue: "Условная стоимость", lifecycle: "Жизненный цикл", expires: "Срок действия", receiverAcceptance: "Принятие получателем", required: "Обязательно", authorityValue: "Стоимость полномочия", zeroValue: "Нулевая стоимость", action: "Действие", scope: "Область", remainingUses: "Осталось использований", delegation: "Делегирование", title: "Приватная отправка", subtitle: "Получатель, объект и авторизация", amount: "Сумма", available: "Доступно: {value}", transferObject: "Передаваемый объект", recipient: "Получатель или приватный запрос", recipientPlaceholder: "Вставьте или отсканируйте приватный запрос", itemLabel: { asset: "Актив", voucher: "Ваучер", permission: "Полномочие" }, note: "Приватная заметка", optional: "необязательно", notePlaceholder: "Для чего это?", cancel: "Отмена", review: "Проверить отправку", reviewTitle: "Проверка отправки", reviewSubtitle: "Проверьте перед авторизацией", family: "Семейство объекта", familyName: { asset: "Актив", voucher: "Ваучер", permission: "Полномочие" }, item: "Объект", recipientShort: "Получатель", from: "Из кошелька", fee: "Комиссия", feeAtAuthorization: "При авторизации", notApplicable: "Не применяется", noteShort: "Заметка", submit: { asset: "Отправить актив", voucher: "Отправить ваучер", permission: "Отправить полномочие" }, sent: { asset: "Актив отправлен", voucher: "Ваучер отправлен", permission: "Полномочие отправлено" }, settlementPending: "Ожидание окончательного расчёта", settling: "Отправлено · выполняется", value: "Значение", nextUpdate: "Следующее обновление", automatic: "Автоматически", viewHistory: "Открыть историю", empty: { asset: "Нет доступных для отправки активов", voucher: "Нет передаваемых ваучеров", permission: "Нет передаваемых полномочий" }
      },
      exchange: {
        executionModel: "Модель исполнения", executionOrderBook: "Спотовая книга заявок", executionSolver: "Кроссчейн solver-intent", title: "Создать запрос обмена", sourceAsset: "Исходный актив", amount: "Сумма", destinationAsset: "Получаемый актив", orderType: "Тип заявки", market: "Рыночная", limit: "Лимитная", limitPrice: "Лимитная цена", limitPricePlaceholder: "Введите положительную цену", recipient: "Получатель", recipientPlaceholder: "Адрес в целевой цепочке", refundAddress: "Адрес возврата", refundPlaceholder: "Адрес возврата в исходной цепочке", slippage: "Допуск проскальзывания", deadline: "Срок котировки", review: "Проверить запрос", reviewTitle: "Проверка запроса обмена", pair: "Спотовая пара", route: "Маршрут", mode: "Режим", exactInput: "Точная входная сумма", from: "Отдаёте", to: "Получаете", rate: "Курс", expectedOutput: "Ожидаемый выход", minimumReceived: "Минимум к получению", fee: "Комиссии", eta: "Расчётное время", depositAddress: "Депозитный адрес", executionStatus: "Статус исполнения", marketState: "Состояние рынка", connectorBoundary: "Коннектор провайдера не зарегистрирован. Курс, комиссии, выход, депозитный адрес и статус исполнения недоступны.", editRequest: "Изменить запрос", amountError: "Введите сумму в пределах доступных {balance} {unit}.", limitPriceError: "Введите положительную лимитную цену.", addressError: "Укажите адрес получателя и адрес возврата в исходной цепочке."
      }
    },
    de: {
      send: { title: "Privat senden", subtitle: "Empfänger, Objekt und Autorisierung", recipient: "Empfänger oder private Anfrage", amount: "Betrag", review: "Sendung prüfen", cancel: "Abbrechen", familyName: { asset: "Asset", voucher: "Gutschein", permission: "Berechtigung" }, itemLabel: { asset: "Asset", voucher: "Gutschein", permission: "Berechtigung" }, empty: { asset: "Keine übertragbaren Assets", voucher: "Keine übertragbaren Gutscheine", permission: "Keine übertragbaren Berechtigungen" } },
      exchange: { executionModel: "Ausführungsmodell", executionOrderBook: "Spot-Orderbuch", executionSolver: "Cross-Chain-Solver-Intent", title: "Exchange-Anfrage erstellen", sourceAsset: "Quell-Asset", destinationAsset: "Ziel-Asset", amount: "Betrag", orderType: "Ordertyp", market: "Markt", limit: "Limit", recipient: "Empfänger", refundAddress: "Rückerstattungsadresse", slippage: "Slippage-Limit", deadline: "Angebotsfrist", review: "Anfrage prüfen", reviewTitle: "Exchange-Anfrage prüfen", quote: "Angebot", minimumReceived: "Mindestens erhalten", fee: "Gebühren", eta: "Geschätzte Zeit", editRequest: "Anfrage bearbeiten" }
    },
    fr: {
      send: { title: "Envoyer en privé", subtitle: "Destinataire, objet et autorisation", recipient: "Destinataire ou demande privée", amount: "Montant", review: "Vérifier l’envoi", cancel: "Annuler", familyName: { asset: "Actif", voucher: "Bon", permission: "Permission" }, itemLabel: { asset: "Actif", voucher: "Bon", permission: "Permission" }, empty: { asset: "Aucun actif transférable", voucher: "Aucun bon transférable", permission: "Aucune permission transférable" } },
      exchange: { executionModel: "Modèle d’exécution", executionOrderBook: "Carnet spot", executionSolver: "Intention solveur inter-chaînes", title: "Créer une demande d’échange", sourceAsset: "Actif source", destinationAsset: "Actif de destination", amount: "Montant", orderType: "Type d’ordre", market: "Marché", limit: "Limite", recipient: "Destinataire", refundAddress: "Adresse de remboursement", slippage: "Tolérance au glissement", deadline: "Échéance du devis", review: "Vérifier la demande", reviewTitle: "Vérifier la demande d’échange", quote: "Devis", minimumReceived: "Minimum reçu", fee: "Frais", eta: "Durée estimée", editRequest: "Modifier la demande" }
    },
    es: {
      send: { title: "Enviar en privado", subtitle: "Destinatario, objeto y autorización", recipient: "Destinatario o solicitud privada", amount: "Importe", review: "Revisar envío", cancel: "Cancelar", familyName: { asset: "Activo", voucher: "Vale", permission: "Permiso" }, itemLabel: { asset: "Activo", voucher: "Vale", permission: "Permiso" }, empty: { asset: "No hay activos transferibles", voucher: "No hay vales transferibles", permission: "No hay permisos transferibles" } },
      exchange: { executionModel: "Modelo de ejecución", executionOrderBook: "Libro spot", executionSolver: "Intent de solver entre cadenas", title: "Crear solicitud de exchange", sourceAsset: "Activo de origen", destinationAsset: "Activo de destino", amount: "Importe", orderType: "Tipo de orden", market: "Mercado", limit: "Límite", recipient: "Destinatario", refundAddress: "Dirección de reembolso", slippage: "Límite de deslizamiento", deadline: "Plazo de cotización", review: "Revisar solicitud", reviewTitle: "Revisar solicitud de exchange", quote: "Cotización", minimumReceived: "Mínimo recibido", fee: "Comisiones", eta: "Tiempo estimado", editRequest: "Editar solicitud" }
    },
    pt: {
      send: { title: "Enviar em privado", subtitle: "Destinatário, objeto e autorização", recipient: "Destinatário ou pedido privado", amount: "Montante", review: "Rever envio", cancel: "Cancelar", familyName: { asset: "Ativo", voucher: "Voucher", permission: "Permissão" }, itemLabel: { asset: "Ativo", voucher: "Voucher", permission: "Permissão" }, empty: { asset: "Sem ativos transferíveis", voucher: "Sem vouchers transferíveis", permission: "Sem permissões transferíveis" } },
      exchange: { executionModel: "Modelo de execução", executionOrderBook: "Livro spot", executionSolver: "Intent de solver entre cadeias", title: "Criar pedido de exchange", sourceAsset: "Ativo de origem", destinationAsset: "Ativo de destino", amount: "Montante", orderType: "Tipo de ordem", market: "Mercado", limit: "Limite", recipient: "Destinatário", refundAddress: "Endereço de reembolso", slippage: "Limite de slippage", deadline: "Prazo da cotação", review: "Rever pedido", reviewTitle: "Rever pedido de exchange", quote: "Cotação", minimumReceived: "Mínimo recebido", fee: "Taxas", eta: "Tempo estimado", editRequest: "Editar pedido" }
    },
    tr: {
      send: { title: "Özel gönder", subtitle: "Alıcı, nesne ve yetkilendirme", recipient: "Alıcı veya özel istek", amount: "Tutar", review: "Gönderimi incele", cancel: "İptal", familyName: { asset: "Varlık", voucher: "Kupon", permission: "İzin" }, itemLabel: { asset: "Varlık", voucher: "Kupon", permission: "İzin" }, empty: { asset: "Aktarılabilir varlık yok", voucher: "Aktarılabilir kupon yok", permission: "Aktarılabilir izin yok" } },
      exchange: { executionModel: "Yürütme modeli", executionOrderBook: "Spot emir defteri", executionSolver: "Zincirler arası solver intent", title: "Exchange isteği oluştur", sourceAsset: "Kaynak varlık", destinationAsset: "Hedef varlık", amount: "Tutar", orderType: "Emir türü", market: "Piyasa", limit: "Limit", recipient: "Alıcı", refundAddress: "İade adresi", slippage: "Kayma sınırı", deadline: "Teklif süresi", review: "İsteği incele", reviewTitle: "Exchange isteğini incele", quote: "Teklif", minimumReceived: "Minimum alınacak", fee: "Ücretler", eta: "Tahmini süre", editRequest: "İsteği düzenle" }
    },
    ja: {
      send: { title: "非公開で送信", subtitle: "受取人、オブジェクト、承認", recipient: "受取人または非公開リクエスト", amount: "数量", review: "送信を確認", cancel: "キャンセル", familyName: { asset: "資産", voucher: "バウチャー", permission: "権限" }, itemLabel: { asset: "資産", voucher: "バウチャー", permission: "権限" }, empty: { asset: "送信可能な資産はありません", voucher: "送信可能なバウチャーはありません", permission: "送信可能な権限はありません" } },
      exchange: { executionModel: "実行モデル", executionOrderBook: "スポット注文板", executionSolver: "クロスチェーン Solver Intent", title: "交換リクエストを作成", sourceAsset: "送信元資産", destinationAsset: "受取資産", amount: "数量", orderType: "注文タイプ", market: "成行", limit: "指値", recipient: "受取人", refundAddress: "返金先アドレス", slippage: "スリッページ上限", deadline: "見積期限", review: "リクエストを確認", reviewTitle: "交換リクエストを確認", quote: "見積", minimumReceived: "最小受取額", fee: "手数料", eta: "推定時間", editRequest: "リクエストを編集" }
    },
    ko: {
      send: { title: "비공개로 보내기", subtitle: "수신자, 객체 및 승인", recipient: "수신자 또는 비공개 요청", amount: "수량", review: "보내기 검토", cancel: "취소", familyName: { asset: "자산", voucher: "바우처", permission: "권한" }, itemLabel: { asset: "자산", voucher: "바우처", permission: "권한" }, empty: { asset: "전송 가능한 자산 없음", voucher: "전송 가능한 바우처 없음", permission: "전송 가능한 권한 없음" } },
      exchange: { executionModel: "실행 모델", executionOrderBook: "현물 오더북", executionSolver: "크로스체인 Solver Intent", title: "Exchange 요청 만들기", sourceAsset: "원본 자산", destinationAsset: "대상 자산", amount: "수량", orderType: "주문 유형", market: "시장가", limit: "지정가", recipient: "수신자", refundAddress: "환불 주소", slippage: "슬리피지 한도", deadline: "견적 기한", review: "요청 검토", reviewTitle: "Exchange 요청 검토", quote: "견적", minimumReceived: "최소 수령액", fee: "수수료", eta: "예상 시간", editRequest: "요청 편집" }
    },
    "zh-Hans": {
      send: { title: "私密发送", subtitle: "收件人、对象与授权", recipient: "收件人或私密请求", amount: "数量", review: "检查发送", cancel: "取消", familyName: { asset: "资产", voucher: "凭证", permission: "权限" }, itemLabel: { asset: "资产", voucher: "凭证", permission: "权限" }, empty: { asset: "没有可转移资产", voucher: "没有可转移凭证", permission: "没有可转移权限" } },
      exchange: { executionModel: "执行模型", executionOrderBook: "现货订单簿", executionSolver: "跨链 Solver Intent", title: "创建兑换请求", sourceAsset: "源资产", destinationAsset: "目标资产", amount: "数量", orderType: "订单类型", market: "市价", limit: "限价", recipient: "收件人", refundAddress: "退款地址", slippage: "滑点上限", deadline: "报价期限", review: "检查请求", reviewTitle: "检查兑换请求", quote: "报价", minimumReceived: "最少收到", fee: "费用", eta: "预计时间", editRequest: "编辑请求" }
    }
  };

  function merge(base, overrides) {
    if (!overrides || typeof overrides !== "object" || Array.isArray(overrides)) return overrides ?? base;
    return Object.fromEntries(Object.entries(base).map(([key, value]) => [
      key,
      value && typeof value === "object" && !Array.isArray(value)
        ? merge(value, overrides[key])
        : overrides[key] ?? value
    ]));
  }

  i18n.extendLocale("en", english);
  i18n.languages().filter(({ id }) => id !== "en").forEach(({ id }) => {
    i18n.extendLocale(id, merge(english, translations[id] || {}));
  });
})();
