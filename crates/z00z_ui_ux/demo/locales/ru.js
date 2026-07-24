"use strict";

window.Z00ZI18n.registerLocale("ru", {
  common: {
    unavailable: "Недоступно",
    readOnly: "Только чтение",
    chain: "Цепочка",
    chainLocked: "Выберите внимательно. После создания кошелька цепочку изменить нельзя.",
    on: "Вкл.",
    off: "Выкл.",
    close: "Закрыть",
    back: "Назад"
  },
  app: {
    documentTitle: "Z00Z Wallet — интерактивная концепция",
    menu: "Меню",
    wallets: "Кошельки",
    network: "Сеть",
    addWallet: "Добавить кошелёк",
    removeWallet: "Удалить кошелёк",
    settings: "Настройки",
    home: "Главная",
    homeContext: "Ваши приватные средства в одном месте",
    settingsContext: "Настройки приложения",
    walletContext: "Кошелёк {wallet}",
    interactiveConcept: "Интерактивная концепция · без реальных средств",
    conceptBuild: "Концепция 0.4 · без реальных средств",
    logOut: "Выйти",
    general: "Общие",
    language: "Язык",
    languageHelp: "Язык, используемый во всём приложении кошелька",
    notifications: "Уведомления",
    notificationsHelp: "Показывать обновления кошелька и необходимые действия",
    regionalFormat: "Региональный формат",
    regionalFormatHelp: "Независимо от языка задаёт формат дат, чисел и десятичных разделителей",
    timeZone: "Часовой пояс",
    timeZoneHelp: "Метки времени хранятся в UTC и показываются в этом часовом поясе",
    networkUnits: "Сетевые единицы",
    networkUnitsHelp: "Скорость сети измеряется в десятичных битах в секунду",
    decimalBitrate: "Десятичные биты в секунду",
    translationCoverage: "Покрытие переводами",
    translationCoverageHelp: "{count} языковых каталогов синхронизированы с английскими ключами",
    languageChanged: "Язык изменён локально."
  },
  nav: {
    home: "Главная",
    assets: "Активы",
    history: "История",
    swap: "Обмен",
    exchange: "Биржа",
    staking: "Стейкинг",
    backup: "Резервная копия",
    settings: "Настройки"
  },
  network: {
    routeTelemetry: "Телеметрия маршрута",
    carrierTelemetry: "Телеметрия носителя",
    publicationTelemetry: "Телеметрия публикаций"
  },
  walletShell: {
    balanceAvailable: "{value} доступно",
    current: "Текущий",
    scanning: "Сканирование",
    identityAria: "Переключить кошелёк. Текущий кошелёк: {wallet}",
    lockLabel: "Кошелёк {wallet}",
    copyAddress: "Скопировать полный адрес кошелька {wallet}",
    available: "Доступно",
    locked: "Заблокировано",
    pendingIn: "Ожидается вход",
    pendingOut: "Ожидается выход",
    routeTelemetry: "Телеметрия маршрута"
  },
  assets: {
    wallet: "Кошелёк", sections: "Разделы кошелька", sectionAssets: "Активы", sectionAssetsHelp: "Доступная и принадлежащая стоимость", sectionVouchers: "Ваучеры", sectionVouchersHelp: "Условная стоимость", sectionPermissions: "Полномочия", sectionPermissionsHelp: "Ограниченные права",
    family: "Семейство активов", title: "Ваши активы", description: "Нативные средства, выпущенные токены и коллекционные объекты различаются. В «Доступно» входят только доступные для траты нативные средства.", send: "Отправить", moneyTotals: "Денежные итоги", available: "Доступно", readyToUse: "Готово к использованию", receiving: "Получение", sending: "Отправка", waitingToSettle: "Ожидает расчёта",
    filters: "Фильтры активов", all: "Все", filterCoins: "Монеты", filterTokens: "Токены", filterNfts: "NFT", kindCoin: "Монета", kindToken: "Токен", kindCollectible: "NFT", needsReview: "Требует проверки", owned: "Принадлежащие активы", ownedHelp: "Класс, доверие и возможность траты указаны явно", name: "Название", balance: "Баланс", value: "Стоимость", price: "Цена",
    viewDetails: "Посмотреть сведения: {asset}", receiveAsset: "Получить {asset}", sendAsset: "Отправить {asset}", receive: "Получить", noMarketFeed: "Нет рыночной котировки", nativeCatalog: "Нативный актив · доверенный каталог", declaredDomain: "Заявленный домен · нужна проверка", uniqueCollectible: "Уникальный коллекционный объект · метаданные доступны", excludedNotice: "Ваучеры, полномочия, изолированные объекты, ненативные токены, коллекционные и экспериментальные совместимые активы исключены из «Доступно».",
    noVouchers: "Ваучеров пока нет", noVouchersHelp: "Создайте ваучер, когда этому кошельку нужна передаваемая условная стоимость.", createVoucher: "Создать ваучер", noPermissions: "Полномочий пока нет", noPermissionsHelp: "Создайте ограниченное полномочие, когда кошельку нужно передаваемое право.", createPermission: "Создать полномочие"
  },
  history: {
    honestSettlement: "Честный расчёт", title: "Все изменения", description: "Отправка и окончательный расчёт показаны как разные состояния. Откройте запись, чтобы увидеть квитанцию и техническую хронологию.", filters: "Фильтры истории", all: "Все", assets: "Активы", vouchers: "Ваучеры", permissions: "Полномочия", system: "Система", needsAttention: "Требует внимания", search: "Поиск по истории", results: "Результаты истории",
    settling: "В расчёте", settled: "Рассчитано", active: "Активно", ready: "Готово", paymentTo: "Платёж для {recipient}", sentWaiting: "Отправлено · ожидает расчёта", allocationClaimed: "Распределение получено", verifiedClaimWaiting: "Заявка проверена · ожидает расчёта", receivedFrom: "Получено от {sender}", yesterday: "Вчера", travelRefundVoucher: "Ваучер на возврат поездки", offeredReviewBefore: "Предложен · проверьте до {date}", deliveryReceiptAccess: "Доступ к квитанции о доставке", dataAccessUsesRemain: "Доступ к данным · осталось {used} из {total} использований", uses: "{count} использований", jul12: "12 июл.", localBackupCreated: "Создана локальная резервная копия", integrityPassed: "Проверка целостности пройдена", jul10: "10 июл.", transferFrom: "Перевод от {wallet}", jul3: "3 июл.", recoveryCheckCompleted: "Проверка восстановления завершена", localVerificationPassed: "Локальная проверка пройдена", jun30: "30 июн.", jul21: "21 июл.", waitingToSettle: "Ожидает расчёта", minutesAgo: "{count} мин. назад", noMatching: "Нет подходящих записей", tryAnother: "Выберите другой фильтр или поисковый запрос.", details: "Сведения истории", status: "Статус", when: "Когда", fee: "Комиссия", feeIncluded: "Включено", feeNotApplicable: "Не применяется", privacy: "Приватность", privacyValue: "Симуляция цели · не живая телеметрия", carrierChain: "Носитель и цепочка", carrierChainValue: "Цель Reticulum · макет основной сети", technicalDetails: "Технические сведения", idLabel: "ID", lifecycleLabel: "Жизненный цикл", lifecyclePending: "создано → отправлено → принято", lifecycleConfirmed: "создано → отправлено → принято → подтверждено", receiptLabel: "Квитанция", copyReceipt: "Скопировать квитанцию", done: "Готово"
  },
  staking: {
    eyebrow: "Стейкинг кошелька", heading: "Стейкинг из {wallet}", description: "Условия стейкинга и ожидающая стоимость отделены от доступного для траты баланса кошелька.", badge: "Основная сеть · концепт", totals: "Итоги стейкинга", availableToStake: "Доступно для стейкинга", walletValueBefore: "Стоимость кошелька до отправки стейкинга", staked: "В стейкинге", nothingDelegated: "В этом концепте ничего не делегировано", rewards: "Награды", accrualNotSimulated: "Начисление не моделируется", prepare: "Подготовить стейкинг", prepareHelp: "Выберите сумму и проверьте условия валидатора перед авторизацией.", amount: "Сумма", availableBalance: "Доступно: {value}", validator: "Валидатор", validatorPlaceholder: "Выберите после проверки цепочки", review: "Проверить стейкинг", safeguards: "Гарантии стейкинга", validatorStatus: "Статус валидатора", unlockPeriod: "Период разблокировки", notSelected: "Не выбран", notProjected: "Не прогнозируется", notice: "Концепт не оценивает доходность и не скрывает риск блокировки. Стейкинг остаётся в ожидании до расчёта в цепочке."
  },
  settings: {
    sections: "Разделы настроек",
    application: "Приложение",
    connectivity: "Подключения",
    general: "Общие",
    generalHelp: "Язык и уведомления приложения",
    appearance: "Внешний вид",
    appearanceHelp: "Тема, палитра, плотность и подсветка YAML",
    networkPrivacy: "Сеть",
    networkPrivacyHelp: "Приватный маршрут, цепочка и синхронизация",
    networkSections: "Разделы сети",
    overview: "Обзор"
  },
  walletSettings: {
    password: "Пароль кошелька", passwordHelp: "Измените пароль этого кошелька локально. Перед установкой нового пароля требуется текущий пароль.", changePassword: "Изменить пароль",
    changePasswordTitle: "Изменить пароль кошелька", changePasswordSubtitle: "Подтвердите текущий пароль, затем выберите новый", currentPassword: "Текущий пароль", newPassword: "Новый пароль", confirmNewPassword: "Подтвердите новый пароль",
    passwordChangeHint: "Используйте не менее 8 символов. Пароли немедленно очищаются из demo.", changePasswordSubmit: "Изменить пароль", passwordChangedTitle: "Пароль обновлён", passwordChangedResult: "Пароль изменён только в этом локальном концепте. Demo очищает все поля и не хранит секрет.",
    passwordCurrentError: "Введите текущий пароль (не менее 8 символов).", passwordNewError: "Используйте не менее 8 символов для нового пароля.", passwordSameError: "Выберите пароль, отличный от текущего.", passwordMismatchError: "Новые пароли не совпадают."
  },
  reticulum: {
    title: "Телеметрия Reticulum",
    summary: "Локальный обзор данных о носителе и доставке. Он помогает оценить доступность, не превращая транспортные метаданные в пользовательскую аналитику.",
    localCapability: "Локальная возможность недоступна",
    localCapabilityHelp: "В demo кошелька не зарегистрирован локальный bridge статуса Reticulum. Интерфейс не имитирует живые данные носителя.",
    tabs: {
      overview: "Обзор",
      node: "Узел",
      interfaces: "Интерфейсы",
      radio: "Радио",
      entrypoints: "Точки входа",
      paths: "Пути",
      links: "Связи",
      probes: "Проверки"
    }
  },
  aggregators: {
    title: "Телеметрия агрегаторов",
    summary: "Данные служб и публикаций для агрегации доступны только для чтения. Кошелёк никогда не передаёт сюда ключи, seed-фразы или секреты политик.",
    localCapability: "Локальная возможность недоступна",
    localCapabilityHelp: "Кошелёк не зарегистрировал bridge к снимку статуса агрегаторов, поэтому страница корректно показывает недоступность.",
    tabs: { overview: "Обзор" }
  },
  onionnet: {
    title: "Телеметрия OnionNet",
    summary: "Обзор детерминированного состояния control plane, локальных наблюдений и агрегированного synthetic health с явными границами данных. Это workspace не меняет маршрут или политику приватности.",
    localCapability: "Локальная возможность недоступна",
    localCapabilityHelp: "В demo кошелька не зарегистрирован bridge статуса OnionNet. Интерфейс не имитирует живые значения маршрута, топологии или приватности.",
    tabs: {
      overview: "Обзор",
      epoch: "Эпоха",
      privacy: "Порог приватности",
      transport: "Транспорт",
      queues: "Очереди и повторы",
      probation: "Проверочный период",
      ingress: "Граница входа"
    }
  },
  help: {
    title: "Справка", openGlobal: "Открыть справку приложения", openContext: "Справка по этому экрану",
    close: "Закрыть справку", contents: "Содержание", section: "Раздел справки {current} из {total}",
    unavailable: "Справка для этого экрана недоступна."
  },
  status: {
    up: "Активен",
    down: "Отключён",
    connecting: "Подключение",
    degraded: "С ограничениями"
  },
  units: {
    bitPerSecond: "{value} бит/с",
    kilobitPerSecond: "{value} кбит/с",
    megabitPerSecond: "{value} Мбит/с"
  }
});
