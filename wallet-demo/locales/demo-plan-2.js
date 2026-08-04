"use strict";

(() => {
  const i18n = window.Z00ZI18n;
  if (!i18n?.extendLocale) throw new Error("DEMO-PLAN-2 labels must load after the locale catalogues.");

  const english = Object.freeze({
    actions: {
      openDetails: "Open details",
      review: "Review",
      retry: "Retry",
      search: "Search",
      accept: "Accept",
      reject: "Reject",
      remove: "Remove",
      delete: "Delete",
      block: "Block",
      report: "Report",
      export: "Export",
      clear: "Clear",
      back: "Back"
    },
    states: {
      roadmapPreview: "Roadmap preview",
      concept: "Concept",
      target: "Target",
      available: "Available",
      degraded: "Degraded",
      unavailable: "Unavailable",
      loading: "Loading",
      empty: "Empty",
      malformed: "Malformed",
      error: "Error",
      pending: "Pending",
      active: "Active",
      approved: "Approved",
      accepted: "Accepted",
      rejected: "Rejected",
      expired: "Expired",
      expiring: "Expiring",
      revoked: "Revoked",
      localOnly: "Local only"
    },
    palette: {
      label: "Palette",
      help: "Select a palette to apply it immediately.",
      previews: "Application palette previews",
      defaultName: "Z00Z Default",
      corporateName: "Z00Z Corporate"
    },
    about: {
      context: "Z00Z application information and release updates",
      productVersion: "Z00Z Wallet v{version}",
      summary: "Private wallet and protocol workspace. This demo defines the desktop and mobile UX target for the future Rust and Tauri application.",
      linksLabel: "Z00Z information and external links",
      privacyPolicy: "Privacy Policy",
      termsOfUse: "Terms of Use",
      visitWebsite: "Visit Z00Z Website",
      visitGitHub: "Visit Z00Z GitHub repository",
      currentVersion: "You are using the current demo version {version}. A packaged application will verify a signed release manifest.",
      checkForUpdates: "Check for updates...",
      updateToast: "Version {version} is current for this demo."
    },
    permission: {
      appIdentity: "App identity",
      action: "Action",
      objectFamily: "Object family",
      exactScope: "Exact scope",
      uses: "Uses",
      expiry: "Expiry",
      delegation: "Delegation",
      value: "Value",
      feePath: "Fee path",
      dataDisclosed: "Data disclosed",
      revokeBehavior: "Revoke behavior",
      reauth: "Re-auth",
      confirmations: "Permission review confirmations",
      confirmScope: "Confirm the exact displayed scope",
      acknowledgeReauth: "Acknowledge Wallet re-auth",
      acceptIntent: "Accept bounded intent"
    },
    aria: {
      extensionCatalogue: "Curated Extension catalogue",
      extensionConnections: "Extension connections",
      extensionPermissions: "Extension permissions",
      extensionActivity: "Extension activity",
      localContacts: "Local contacts",
      watcherControls: "Deterministic Watchers controls",
      explorerControls: "Deterministic Explorer controls",
      explorerDetail: "Explorer detail level",
      palettePreviews: "Application palette previews"
    }
  });

  const translations = Object.freeze({
    ru: {
      actions: { openDetails: "Открыть сведения", review: "Проверить", retry: "Повторить", search: "Поиск", accept: "Принять", reject: "Отклонить", remove: "Удалить", delete: "Удалить", block: "Заблокировать", report: "Пожаловаться", export: "Экспортировать", clear: "Очистить", back: "Назад" },
      states: { roadmapPreview: "План развития", concept: "Концепция", target: "Цель", available: "Доступно", degraded: "Ограничено", unavailable: "Недоступно", loading: "Загрузка", empty: "Пусто", malformed: "Некорректные данные", error: "Ошибка", pending: "Ожидает", active: "Активно", approved: "Одобрено", accepted: "Принято", rejected: "Отклонено", expired: "Истекло", expiring: "Истекает", revoked: "Отозвано", localOnly: "Только локально" },
      palette: { label: "Палитра", help: "Выберите палитру, чтобы применить её сразу.", previews: "Палитры приложения" },
      about: { context: "Информация о приложении Z00Z и обновлениях", productVersion: "Z00Z Wallet v{version}", summary: "Приватный кошелёк и рабочее пространство протокола. Это демо задаёт UX для будущего приложения Rust и Tauri на компьютерах и мобильных устройствах.", linksLabel: "Информация Z00Z и внешние ссылки", privacyPolicy: "Политика конфиденциальности", termsOfUse: "Условия использования", visitWebsite: "Перейти на сайт Z00Z", visitGitHub: "Открыть репозиторий Z00Z на GitHub", currentVersion: "Установлена текущая демоверсия {version}. Готовое приложение будет проверять подписанный манифест выпуска.", checkForUpdates: "Проверить обновления...", updateToast: "Для этого демо версия {version} актуальна." },
      permission: { appIdentity: "Приложение", action: "Действие", objectFamily: "Тип объекта", exactScope: "Точная область", uses: "Использования", expiry: "Срок действия", delegation: "Делегирование", value: "Значение", feePath: "Комиссия", dataDisclosed: "Раскрываемые данные", revokeBehavior: "Поведение при отзыве", reauth: "Повторная авторизация", confirmations: "Подтверждения разрешения", confirmScope: "Подтвердить показанную область", acknowledgeReauth: "Подтвердить повторную авторизацию Кошелька", acceptIntent: "Принять ограниченное намерение" },
      aria: { extensionCatalogue: "Каталог отобранных расширений", extensionConnections: "Подключения расширений", extensionPermissions: "Разрешения расширений", extensionActivity: "Активность расширений", localContacts: "Локальные контакты", watcherControls: "Детерминированные настройки Watchers", explorerControls: "Детерминированные настройки Explorer", explorerDetail: "Уровень сведений Explorer" }
    },
    fr: {
      actions: { openDetails: "Ouvrir les détails", review: "Examiner", retry: "Réessayer", search: "Rechercher", accept: "Accepter", reject: "Refuser", remove: "Retirer", delete: "Supprimer", block: "Bloquer", report: "Signaler", export: "Exporter", clear: "Effacer", back: "Retour" },
      states: { roadmapPreview: "Aperçu de feuille de route", concept: "Concept", target: "Cible", available: "Disponible", degraded: "Dégradé", unavailable: "Indisponible", loading: "Chargement", empty: "Vide", malformed: "Invalide", error: "Erreur", pending: "En attente", active: "Actif", approved: "Approuvé", accepted: "Accepté", rejected: "Refusé", expired: "Expiré", expiring: "Expire bientôt", revoked: "Révoqué", localOnly: "Local uniquement" },
      palette: { label: "Palette", help: "Sélectionnez une palette pour l’appliquer immédiatement.", previews: "Palettes de l’application" },
      about: { context: "Informations sur l’application Z00Z et ses mises à jour", productVersion: "Z00Z Wallet v{version}", summary: "Portefeuille privé et espace de travail du protocole. Cette démo définit la cible UX de la future application Rust et Tauri sur ordinateur et mobile.", linksLabel: "Informations Z00Z et liens externes", privacyPolicy: "Politique de confidentialité", termsOfUse: "Conditions d’utilisation", visitWebsite: "Visiter le site Z00Z", visitGitHub: "Voir le dépôt Z00Z sur GitHub", currentVersion: "Vous utilisez la version de démonstration actuelle {version}. L’application installée vérifiera un manifeste de version signé.", checkForUpdates: "Rechercher des mises à jour...", updateToast: "La version {version} est à jour pour cette démo." },
      permission: { appIdentity: "Identité de l’app", action: "Action", objectFamily: "Famille d’objet", exactScope: "Portée exacte", uses: "Utilisations", expiry: "Expiration", delegation: "Délégation", value: "Valeur", feePath: "Frais", dataDisclosed: "Données divulguées", revokeBehavior: "Effet de la révocation", reauth: "Réauthentification", confirmations: "Confirmations de l’autorisation", confirmScope: "Confirmer la portée affichée", acknowledgeReauth: "Confirmer la réauthentification du portefeuille", acceptIntent: "Accepter l’intention limitée" },
      aria: { extensionCatalogue: "Catalogue d’extensions sélectionnées", extensionConnections: "Connexions d’extensions", extensionPermissions: "Autorisations des extensions", extensionActivity: "Activité des extensions", localContacts: "Contacts locaux", watcherControls: "Contrôles Watchers déterministes", explorerControls: "Contrôles Explorer déterministes", explorerDetail: "Niveau de détail Explorer" }
    },
    de: {
      actions: { openDetails: "Details öffnen", review: "Prüfen", retry: "Erneut versuchen", search: "Suchen", accept: "Annehmen", reject: "Ablehnen", remove: "Entfernen", delete: "Löschen", block: "Blockieren", report: "Melden", export: "Exportieren", clear: "Leeren", back: "Zurück" },
      states: { roadmapPreview: "Roadmap-Vorschau", concept: "Konzept", target: "Ziel", available: "Verfügbar", degraded: "Eingeschränkt", unavailable: "Nicht verfügbar", loading: "Laden", empty: "Leer", malformed: "Ungültig", error: "Fehler", pending: "Ausstehend", active: "Aktiv", approved: "Genehmigt", accepted: "Angenommen", rejected: "Abgelehnt", expired: "Abgelaufen", expiring: "Läuft ab", revoked: "Widerrufen", localOnly: "Nur lokal" },
      palette: { label: "Palette", help: "Wählen Sie eine Palette aus, um sie sofort anzuwenden.", previews: "Anwendungspaletten" },
      about: { context: "Informationen zur Z00Z-Anwendung und zu Updates", productVersion: "Z00Z Wallet v{version}", summary: "Privates Wallet und Protokoll-Arbeitsbereich. Diese Demo definiert das UX-Ziel der künftigen Rust- und Tauri-Anwendung für Desktop und Mobilgeräte.", linksLabel: "Z00Z-Informationen und externe Links", privacyPolicy: "Datenschutzerklärung", termsOfUse: "Nutzungsbedingungen", visitWebsite: "Z00Z-Website besuchen", visitGitHub: "Z00Z-Repository auf GitHub öffnen", currentVersion: "Sie verwenden die aktuelle Demoversion {version}. Die installierte Anwendung wird ein signiertes Release-Manifest prüfen.", checkForUpdates: "Nach Updates suchen...", updateToast: "Version {version} ist für diese Demo aktuell." },
      permission: { appIdentity: "App-Identität", action: "Aktion", objectFamily: "Objektfamilie", exactScope: "Genauer Umfang", uses: "Nutzungen", expiry: "Ablauf", delegation: "Delegierung", value: "Wert", feePath: "Gebührenpfad", dataDisclosed: "Offengelegte Daten", revokeBehavior: "Widerrufsverhalten", reauth: "Erneute Anmeldung", confirmations: "Berechtigungsbestätigungen", confirmScope: "Angezeigten Umfang bestätigen", acknowledgeReauth: "Wallet-Anmeldung bestätigen", acceptIntent: "Begrenzte Absicht annehmen" },
      aria: { extensionCatalogue: "Katalog ausgewählter Erweiterungen", extensionConnections: "Erweiterungsverbindungen", extensionPermissions: "Erweiterungsberechtigungen", extensionActivity: "Erweiterungsaktivität", localContacts: "Lokale Kontakte", watcherControls: "Deterministische Watchers-Steuerung", explorerControls: "Deterministische Explorer-Steuerung", explorerDetail: "Explorer-Detailstufe" }
    },
    es: {
      actions: { openDetails: "Abrir detalles", review: "Revisar", retry: "Reintentar", search: "Buscar", accept: "Aceptar", reject: "Rechazar", remove: "Eliminar", delete: "Borrar", block: "Bloquear", report: "Denunciar", export: "Exportar", clear: "Limpiar", back: "Volver" },
      states: { roadmapPreview: "Vista previa de hoja de ruta", concept: "Concepto", target: "Objetivo", available: "Disponible", degraded: "Degradado", unavailable: "No disponible", loading: "Cargando", empty: "Vacío", malformed: "No válido", error: "Error", pending: "Pendiente", active: "Activo", approved: "Aprobado", accepted: "Aceptado", rejected: "Rechazado", expired: "Caducado", expiring: "Próximo a caducar", revoked: "Revocado", localOnly: "Solo local" },
      palette: { label: "Paleta", help: "Seleccione una paleta para aplicarla inmediatamente.", previews: "Paletas de la aplicación" },
      about: { context: "Información de la aplicación Z00Z y actualizaciones", productVersion: "Z00Z Wallet v{version}", summary: "Cartera privada y espacio de trabajo del protocolo. Esta demo define el objetivo UX de la futura aplicación Rust y Tauri para escritorio y móvil.", linksLabel: "Información de Z00Z y enlaces externos", privacyPolicy: "Política de privacidad", termsOfUse: "Términos de uso", visitWebsite: "Visitar el sitio web de Z00Z", visitGitHub: "Abrir el repositorio Z00Z en GitHub", currentVersion: "Está utilizando la versión de demostración actual {version}. La aplicación instalada verificará un manifiesto de versión firmado.", checkForUpdates: "Buscar actualizaciones...", updateToast: "La versión {version} está actualizada para esta demo." },
      permission: { appIdentity: "Identidad de la app", action: "Acción", objectFamily: "Familia de objeto", exactScope: "Ámbito exacto", uses: "Usos", expiry: "Caducidad", delegation: "Delegación", value: "Valor", feePath: "Ruta de comisión", dataDisclosed: "Datos divulgados", revokeBehavior: "Efecto de revocación", reauth: "Reautenticación", confirmations: "Confirmaciones del permiso", confirmScope: "Confirmar el ámbito mostrado", acknowledgeReauth: "Confirmar reautenticación de la cartera", acceptIntent: "Aceptar intención limitada" },
      aria: { extensionCatalogue: "Catálogo de extensiones seleccionadas", extensionConnections: "Conexiones de extensiones", extensionPermissions: "Permisos de extensiones", extensionActivity: "Actividad de extensiones", localContacts: "Contactos locales", watcherControls: "Controles deterministas de Watchers", explorerControls: "Controles deterministas de Explorer", explorerDetail: "Nivel de detalle de Explorer" }
    },
    pt: {
      actions: { openDetails: "Abrir detalhes", review: "Rever", retry: "Tentar novamente", search: "Pesquisar", accept: "Aceitar", reject: "Rejeitar", remove: "Remover", delete: "Eliminar", block: "Bloquear", report: "Denunciar", export: "Exportar", clear: "Limpar", back: "Voltar" },
      states: { roadmapPreview: "Pré-visualização do roteiro", concept: "Conceito", target: "Objetivo", available: "Disponível", degraded: "Degradado", unavailable: "Indisponível", loading: "A carregar", empty: "Vazio", malformed: "Inválido", error: "Erro", pending: "Pendente", active: "Ativo", approved: "Aprovado", accepted: "Aceite", rejected: "Rejeitado", expired: "Expirado", expiring: "A expirar", revoked: "Revogado", localOnly: "Apenas local" },
      palette: { label: "Paleta", help: "Selecione uma paleta para aplicá-la imediatamente.", previews: "Paletas da aplicação" },
      about: { context: "Informações da aplicação Z00Z e atualizações", productVersion: "Z00Z Wallet v{version}", summary: "Carteira privada e espaço de trabalho do protocolo. Esta demonstração define o objetivo UX da futura aplicação Rust e Tauri para computador e dispositivos móveis.", linksLabel: "Informações Z00Z e ligações externas", privacyPolicy: "Política de privacidade", termsOfUse: "Termos de utilização", visitWebsite: "Visitar o site Z00Z", visitGitHub: "Abrir o repositório Z00Z no GitHub", currentVersion: "Está a utilizar a versão de demonstração atual {version}. A aplicação instalada verificará um manifesto de versão assinado.", checkForUpdates: "Procurar atualizações...", updateToast: "A versão {version} está atualizada para esta demonstração." },
      permission: { appIdentity: "Identidade da app", action: "Ação", objectFamily: "Família do objeto", exactScope: "Âmbito exato", uses: "Utilizações", expiry: "Validade", delegation: "Delegação", value: "Valor", feePath: "Caminho da taxa", dataDisclosed: "Dados divulgados", revokeBehavior: "Efeito da revogação", reauth: "Reautenticação", confirmations: "Confirmações da permissão", confirmScope: "Confirmar o âmbito apresentado", acknowledgeReauth: "Confirmar reautenticação da carteira", acceptIntent: "Aceitar intenção limitada" },
      aria: { extensionCatalogue: "Catálogo de extensões selecionadas", extensionConnections: "Ligações de extensões", extensionPermissions: "Permissões de extensões", extensionActivity: "Atividade de extensões", localContacts: "Contactos locais", watcherControls: "Controlos determinísticos de Watchers", explorerControls: "Controlos determinísticos de Explorer", explorerDetail: "Nível de detalhe do Explorer" }
    },
    ko: {
      actions: { openDetails: "세부 정보 열기", review: "검토", retry: "다시 시도", search: "검색", accept: "수락", reject: "거부", remove: "제거", delete: "삭제", block: "차단", report: "신고", export: "내보내기", clear: "지우기", back: "뒤로" },
      states: { roadmapPreview: "로드맵 미리보기", concept: "개념", target: "목표", available: "사용 가능", degraded: "제한됨", unavailable: "사용 불가", loading: "로딩 중", empty: "비어 있음", malformed: "잘못됨", error: "오류", pending: "대기 중", active: "활성", approved: "승인됨", accepted: "수락됨", rejected: "거부됨", expired: "만료됨", expiring: "만료 예정", revoked: "취소됨", localOnly: "로컬 전용" },
      palette: { label: "팔레트", help: "팔레트를 선택하면 즉시 적용됩니다.", previews: "앱 팔레트" },
      about: { context: "Z00Z 애플리케이션 정보 및 업데이트", productVersion: "Z00Z Wallet v{version}", summary: "개인 지갑 및 프로토콜 작업 공간입니다. 이 데모는 향후 Rust 및 Tauri 데스크톱·모바일 애플리케이션의 UX 목표를 정의합니다.", linksLabel: "Z00Z 정보 및 외부 링크", privacyPolicy: "개인정보 처리방침", termsOfUse: "이용 약관", visitWebsite: "Z00Z 웹사이트 방문", visitGitHub: "GitHub에서 Z00Z 저장소 열기", currentVersion: "현재 데모 버전 {version}을 사용 중입니다. 설치형 애플리케이션은 서명된 릴리스 매니페스트를 확인합니다.", checkForUpdates: "업데이트 확인...", updateToast: "이 데모의 버전 {version}은 최신입니다." },
      permission: { appIdentity: "앱 신원", action: "작업", objectFamily: "객체 종류", exactScope: "정확한 범위", uses: "사용 횟수", expiry: "만료", delegation: "위임", value: "값", feePath: "수수료 경로", dataDisclosed: "공개 데이터", revokeBehavior: "취소 동작", reauth: "재인증", confirmations: "권한 검토 확인", confirmScope: "표시된 범위 확인", acknowledgeReauth: "지갑 재인증 확인", acceptIntent: "제한된 인텐트 수락" },
      aria: { extensionCatalogue: "선별된 확장 기능 카탈로그", extensionConnections: "확장 기능 연결", extensionPermissions: "확장 기능 권한", extensionActivity: "확장 기능 활동", localContacts: "로컬 연락처", watcherControls: "결정론적 Watchers 제어", explorerControls: "결정론적 Explorer 제어", explorerDetail: "Explorer 세부 수준" }
    },
    tr: {
      actions: { openDetails: "Ayrıntıları aç", review: "İncele", retry: "Yeniden dene", search: "Ara", accept: "Kabul et", reject: "Reddet", remove: "Kaldır", delete: "Sil", block: "Engelle", report: "Bildir", export: "Dışa aktar", clear: "Temizle", back: "Geri" },
      states: { roadmapPreview: "Yol haritası önizlemesi", concept: "Kavram", target: "Hedef", available: "Kullanılabilir", degraded: "Kısıtlı", unavailable: "Kullanılamıyor", loading: "Yükleniyor", empty: "Boş", malformed: "Geçersiz", error: "Hata", pending: "Bekliyor", active: "Etkin", approved: "Onaylandı", accepted: "Kabul edildi", rejected: "Reddedildi", expired: "Süresi doldu", expiring: "Süresi doluyor", revoked: "İptal edildi", localOnly: "Yalnızca yerel" },
      palette: { label: "Palet", help: "Hemen uygulamak için bir palet seçin.", previews: "Uygulama paletleri" },
      about: { context: "Z00Z uygulama bilgileri ve güncellemeleri", productVersion: "Z00Z Wallet v{version}", summary: "Özel cüzdan ve protokol çalışma alanı. Bu demo, gelecekteki Rust ve Tauri masaüstü ve mobil uygulamasının UX hedefini tanımlar.", linksLabel: "Z00Z bilgileri ve harici bağlantılar", privacyPolicy: "Gizlilik Politikası", termsOfUse: "Kullanım Koşulları", visitWebsite: "Z00Z web sitesini ziyaret et", visitGitHub: "Z00Z GitHub deposunu aç", currentVersion: "Güncel demo sürümü {version} kullanılıyor. Paketli uygulama imzalı bir sürüm bildirimini doğrulayacaktır.", checkForUpdates: "Güncellemeleri denetle...", updateToast: "Bu demo için {version} sürümü günceldir." },
      permission: { appIdentity: "Uygulama kimliği", action: "İşlem", objectFamily: "Nesne ailesi", exactScope: "Kesin kapsam", uses: "Kullanımlar", expiry: "Süre", delegation: "Yetki devri", value: "Değer", feePath: "Ücret yolu", dataDisclosed: "Açıklanan veri", revokeBehavior: "İptal davranışı", reauth: "Yeniden kimlik doğrulama", confirmations: "İzin inceleme onayları", confirmScope: "Gösterilen kapsamı onayla", acknowledgeReauth: "Cüzdan yeniden doğrulamasını onayla", acceptIntent: "Sınırlı niyeti kabul et" },
      aria: { extensionCatalogue: "Seçilmiş uzantı kataloğu", extensionConnections: "Uzantı bağlantıları", extensionPermissions: "Uzantı izinleri", extensionActivity: "Uzantı etkinliği", localContacts: "Yerel kişiler", watcherControls: "Belirlenimci Watchers denetimleri", explorerControls: "Belirlenimci Explorer denetimleri", explorerDetail: "Explorer ayrıntı düzeyi" }
    },
    ja: {
      actions: { openDetails: "詳細を開く", review: "確認", retry: "再試行", search: "検索", accept: "承認", reject: "拒否", remove: "削除", delete: "削除", block: "ブロック", report: "報告", export: "エクスポート", clear: "クリア", back: "戻る" },
      states: { roadmapPreview: "ロードマッププレビュー", concept: "コンセプト", target: "対象", available: "利用可能", degraded: "制限あり", unavailable: "利用不可", loading: "読み込み中", empty: "空", malformed: "不正", error: "エラー", pending: "保留中", active: "有効", approved: "承認済み", accepted: "承認", rejected: "拒否", expired: "期限切れ", expiring: "期限間近", revoked: "失効", localOnly: "ローカルのみ" },
      palette: { label: "パレット", help: "パレットを選択するとすぐに適用されます。", previews: "アプリのパレット" },
      about: { context: "Z00Z アプリケーション情報とアップデート", productVersion: "Z00Z Wallet v{version}", summary: "プライベートウォレットとプロトコルのワークスペースです。このデモは、将来の Rust および Tauri デスクトップ・モバイルアプリの UX 目標を定義します。", linksLabel: "Z00Z の情報と外部リンク", privacyPolicy: "プライバシーポリシー", termsOfUse: "利用規約", visitWebsite: "Z00Z ウェブサイトを開く", visitGitHub: "Z00Z GitHub リポジトリを開く", currentVersion: "現在のデモバージョン {version} を使用しています。インストール版アプリは署名済みリリースマニフェストを検証します。", checkForUpdates: "アップデートを確認...", updateToast: "このデモのバージョン {version} は最新です。" },
      permission: { appIdentity: "アプリ識別情報", action: "操作", objectFamily: "オブジェクト種別", exactScope: "正確な範囲", uses: "利用回数", expiry: "有効期限", delegation: "委任", value: "値", feePath: "手数料経路", dataDisclosed: "開示データ", revokeBehavior: "失効時の動作", reauth: "再認証", confirmations: "権限確認", confirmScope: "表示された範囲を確認", acknowledgeReauth: "ウォレット再認証を確認", acceptIntent: "限定インテントを承認" },
      aria: { extensionCatalogue: "選定済み拡張機能カタログ", extensionConnections: "拡張機能接続", extensionPermissions: "拡張機能の権限", extensionActivity: "拡張機能のアクティビティ", localContacts: "ローカル連絡先", watcherControls: "決定的 Watchers 操作", explorerControls: "決定的 Explorer 操作", explorerDetail: "Explorer 詳細レベル" }
    },
    "zh-Hans": {
      actions: { openDetails: "打开详情", review: "审核", retry: "重试", search: "搜索", accept: "接受", reject: "拒绝", remove: "移除", delete: "删除", block: "屏蔽", report: "举报", export: "导出", clear: "清除", back: "返回" },
      states: { roadmapPreview: "路线图预览", concept: "概念", target: "目标", available: "可用", degraded: "受限", unavailable: "不可用", loading: "加载中", empty: "空", malformed: "格式错误", error: "错误", pending: "待处理", active: "有效", approved: "已批准", accepted: "已接受", rejected: "已拒绝", expired: "已过期", expiring: "即将过期", revoked: "已撤销", localOnly: "仅本地" },
      palette: { label: "配色", help: "选择配色后会立即应用。", previews: "应用配色" },
      about: { context: "Z00Z 应用信息和更新", productVersion: "Z00Z Wallet v{version}", summary: "私密钱包和协议工作区。此演示定义未来 Rust 和 Tauri 桌面与移动应用的 UX 目标。", linksLabel: "Z00Z 信息和外部链接", privacyPolicy: "隐私政策", termsOfUse: "使用条款", visitWebsite: "访问 Z00Z 网站", visitGitHub: "打开 Z00Z GitHub 仓库", currentVersion: "您正在使用当前演示版本 {version}。安装版应用将验证已签名的发布清单。", checkForUpdates: "检查更新...", updateToast: "此演示的版本 {version} 已是最新。" },
      permission: { appIdentity: "应用身份", action: "操作", objectFamily: "对象类型", exactScope: "精确范围", uses: "使用次数", expiry: "期限", delegation: "委托", value: "值", feePath: "费用路径", dataDisclosed: "披露数据", revokeBehavior: "撤销行为", reauth: "重新认证", confirmations: "权限审核确认", confirmScope: "确认显示的范围", acknowledgeReauth: "确认钱包重新认证", acceptIntent: "接受受限意图" },
      aria: { extensionCatalogue: "精选扩展目录", extensionConnections: "扩展连接", extensionPermissions: "扩展权限", extensionActivity: "扩展活动", localContacts: "本地联系人", watcherControls: "确定性 Watchers 控件", explorerControls: "确定性 Explorer 控件", explorerDetail: "Explorer 详情级别" }
    }
  });

  const merge = (base, override = {}) => Object.fromEntries(Object.entries(base).map(([key, value]) => [
    key,
    value && typeof value === "object" && !Array.isArray(value)
      ? merge(value, override[key] || {})
      : (override[key] ?? value)
  ]));

  i18n.extendLocale("en", { plan2: english });
  Object.entries(translations).forEach(([language, labels]) => {
    i18n.extendLocale(language, { plan2: merge(english, labels) });
  });
})();
