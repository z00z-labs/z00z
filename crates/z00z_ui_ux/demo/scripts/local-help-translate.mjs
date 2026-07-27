#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const locale = process.argv[2];
const supportedLocales = ["ru", "fr", "de", "es", "pt", "ko", "tr", "ja", "zh-Hans"];

if (!supportedLocales.includes(locale)) {
  throw new Error(`Unsupported local Help translation target: ${locale || "missing"}`);
}

let input = "";
for await (const chunk of process.stdin) input += chunk;
const payload = JSON.parse(input);
if (
  payload.contentType !== "z00z-help-messages-v1"
  || payload.language !== locale
  || payload.sourceLanguage !== "en"
  || !payload.topic
  || !payload.messages
) {
  throw new Error("Invalid local Help translation payload.");
}

const context = { window: {}, Intl };
context.globalThis = context.window;
for (const relativePath of [
  "scripts/port/locale-registry.js",
  "i18n.js",
  "locales/en.js",
  `locales/${locale}.js`,
  "locales/navigation.js"
]) {
  const sourcePath = resolve(demoRoot, relativePath);
  vm.runInNewContext(await readFile(sourcePath, "utf8"), context, { filename: sourcePath });
}
const catalogue = context.window.Z00ZI18n.catalogue(locale);
const nav = (key) => catalogue[`navigation.${key}`] || key;

const TITLE_JOINERS = Object.freeze({
  ru: " — ",
  fr: " — ",
  de: " – ",
  es: " — ",
  pt: " — ",
  ko: " · ",
  tr: " — ",
  ja: "：",
  "zh-Hans": "："
});

const TITLE_TERMS = Object.freeze({
  ru: { details: "сведения", permissionReview: "проверка разрешения", requestReview: "проверка запроса", identityReview: "проверка личности", alertDetails: "сведения об оповещении" },
  fr: { details: "détails", permissionReview: "examen de l’autorisation", requestReview: "examen de la demande", identityReview: "examen de l’identité", alertDetails: "détails de l’alerte" },
  de: { details: "Details", permissionReview: "Berechtigung prüfen", requestReview: "Anfrage prüfen", identityReview: "Identität prüfen", alertDetails: "Warnungsdetails" },
  es: { details: "detalles", permissionReview: "revisión del permiso", requestReview: "revisión de la solicitud", identityReview: "revisión de identidad", alertDetails: "detalles de la alerta" },
  pt: { details: "detalhes", permissionReview: "revisão da permissão", requestReview: "revisão do pedido", identityReview: "revisão da identidade", alertDetails: "detalhes do alerta" },
  ko: { details: "세부 정보", permissionReview: "권한 검토", requestReview: "요청 검토", identityReview: "신원 검토", alertDetails: "알림 세부 정보" },
  tr: { details: "ayrıntılar", permissionReview: "izin incelemesi", requestReview: "istek incelemesi", identityReview: "kimlik incelemesi", alertDetails: "uyarı ayrıntıları" },
  ja: { details: "詳細", permissionReview: "権限の確認", requestReview: "リクエストの確認", identityReview: "本人情報の確認", alertDetails: "アラートの詳細" },
  "zh-Hans": { details: "详情", permissionReview: "权限审核", requestReview: "请求审核", identityReview: "身份审核", alertDetails: "警报详情" }
});

const COPY = Object.freeze({
  ru: {
    headings: ["Использование этого экрана", "Локальное и безопасное поведение"],
    wallet: ["{title}: локальная справка для объектов, требующих явной проверки кошельком.", "Проверьте указанную причину, источник и локальный статус перед любым действием.", "Недоступное действие остаётся заблокированным, пока нативный кошелёк не сообщит безопасный следующий шаг.", "Окончательное решение принимает политика нативного кошелька, а не этот экран.", "Секреты и приватные транспортные данные не попадают в справку."],
    watchers: ["{title}: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.", "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.", "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие.", "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.", "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."],
    explorer: ["{title}: справка о приватном демо Explorer для поддерживаемых публичных идентификаторов.", "Используйте только поддерживаемые публичные идентификаторы контрольных точек, пакетов, оповещений и доказательств.", "Неизвестные, приватные, повреждённые или недоступные идентификаторы блокируются без обращения к кошельку.", "Explorer — интерактивная дорожная карта на локальных фикстурах, а не сервис данных кошелька.", "Локальные балансы, контакты, сообщения, заметки, маршруты и секреты не передаются в Explorer."],
    dapps: ["{title}: справка об ограниченном локальном демо dApps и его разрешениях.", "Проверьте локальные описания, ограниченные намерения и явный результат операции.", "Перед принятием проверьте область, число использований, срок, сумму, комиссию, раскрытие и отзыв.", "dApps — интерактивная дорожная карта: удалённый код, произвольные URL и универсальная подпись не выполняются.", "Принятое намерение заново проверяет Кошелёк; этот экран не изменяет объекты кошелька."],
    messenger: ["{title}: справка о приватном демо координации запросов и передаче в Кошелёк.", "Проверьте локальные сообщения, запросы, квитанции, срок действия и состояния восстановления.", "Принятие запроса создаёт намерение для проверки в Кошельке, но ничего не рассчитывает и не изменяет.", "Messenger — интерактивная дорожная карта краткоживущей ретрансляции, а не постоянный чат в цепочке.", "Открытие, удаление, блокировка или жалоба не меняют состояние расчётов Кошелька."],
    contacts: ["{title}: справка о локальных метках контактов, карточках получателя и проверке изменения личности.", "Проверьте локальные данные контакта, срок, отзыв и доказательства изменения личности.", "Сохранённая метка не доказывает личность или доверие; изменённые данные требуют явной проверки.", "Контакты остаются локальными и не публикуются как граф адресов или присутствия.", "Удаление локального контакта не отзывает внешние полномочия и не изменяет расчёты Кошелька."]
  },
  fr: {
    headings: ["Utiliser cette vue", "Comportement local et sûr"],
    wallet: ["{title} : aide locale pour les objets qui exigent un examen explicite du portefeuille.", "Vérifiez le motif, la source et l’état local indiqués avant toute action.", "Une action indisponible reste bloquée jusqu’à ce que le portefeuille natif fournisse une étape sûre.", "La décision finale appartient à la politique du portefeuille natif, pas à cette vue.", "Les secrets et les données de transport privées n’entrent jamais dans l’aide."],
    watchers: ["{title} : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.", "Consultez les données déterministes de publication sans modifier l’état du réseau.", "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre.", "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.", "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."],
    explorer: ["{title} : aide sur l’aperçu Explorer respectueux de la vie privée pour les identifiants publics pris en charge.", "N’utilisez que les identifiants publics pris en charge pour les points de contrôle, lots, alertes ou preuves.", "Les identifiants inconnus, privés, malformés ou indisponibles échouent sans consulter le portefeuille.", "Explorer est un aperçu de feuille de route fondé sur des données locales, pas un service de données du portefeuille.", "Soldes locaux, contacts, messages, notes, routes et secrets n’entrent jamais dans Explorer."],
    dapps: ["{title} : aide sur l’aperçu dApps local limité et sa frontière d’autorisation.", "Examinez les descriptions locales, intentions limitées et résultats explicites.", "Avant d’accepter, vérifiez portée, utilisations, expiration, valeur, frais, divulgation et révocation.", "dApps est un aperçu de feuille de route : aucun code distant, URL arbitraire ou signature générique n’est exécuté.", "Le portefeuille revalide toute intention acceptée ; cette vue ne modifie pas ses objets."],
    messenger: ["{title} : aide sur l’aperçu privé de coordination des demandes et son transfert vers le portefeuille.", "Examinez les messages, demandes, reçus, expirations et états de récupération locaux.", "Accepter une demande crée une intention à examiner dans le portefeuille sans règlement ni mutation.", "Messenger est un aperçu de feuille de route pour relais éphémères, pas une messagerie permanente sur chaîne.", "Ouvrir, supprimer, bloquer ou signaler ne change jamais l’état de règlement du portefeuille."],
    contacts: ["{title} : aide sur les libellés locaux, cartes de réception et changements d’identité explicites.", "Examinez les données locales, l’expiration, la révocation et les preuves de changement d’identité.", "Un libellé enregistré ne prouve ni identité ni confiance ; toute donnée modifiée exige un examen.", "Les contacts restent locaux et ne sont jamais publiés comme graphe d’adresses ou de présence.", "Supprimer un contact local ne révoque pas les droits externes et ne change pas le règlement du portefeuille."]
  },
  de: {
    headings: ["Diese Ansicht verwenden", "Lokales und sicheres Verhalten"],
    wallet: ["{title}: lokale Hilfe für Objekte, die eine ausdrückliche Wallet-Prüfung benötigen.", "Prüfen Sie vor jeder Aktion den angegebenen Grund, die Quelle und den lokalen Status.", "Eine nicht verfügbare Aktion bleibt gesperrt, bis die native Wallet einen sicheren nächsten Schritt meldet.", "Die native Wallet-Richtlinie entscheidet endgültig, nicht diese Ansicht.", "Geheimnisse und private Transportdaten gelangen nie in die Hilfe."],
    watchers: ["{title}: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.", "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.", "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher.", "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.", "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."],
    explorer: ["{title}: Hilfe zur datenschutzbegrenzten Explorer-Vorschau für unterstützte öffentliche Kennungen.", "Verwenden Sie nur unterstützte öffentliche Kennungen für Prüfpunkte, Stapel, Warnungen oder Nachweise.", "Unbekannte, private, ungültige oder nicht verfügbare Kennungen scheitern ohne Wallet-Abfrage.", "Explorer ist eine Roadmap-Vorschau mit lokalen Fixtures, kein Wallet-Datendienst.", "Lokale Salden, Kontakte, Nachrichten, Notizen, Routen und Geheimnisse gelangen nie in Explorer."],
    dapps: ["{title}: Hilfe zur begrenzten lokalen dApps-Vorschau und ihrer Berechtigungsgrenze.", "Prüfen Sie lokale Beschreibungen, begrenzte Absichten und eindeutige Ergebnisse.", "Prüfen Sie vor Annahme Umfang, Nutzungen, Ablauf, Wert, Gebühren, Offenlegung und Widerruf.", "dApps ist eine Roadmap-Vorschau: Kein entfernter Code, keine beliebige URL und keine generische Signatur wird ausgeführt.", "Die Wallet prüft angenommene Absichten erneut; diese Ansicht verändert keine Wallet-Objekte."],
    messenger: ["{title}: Hilfe zur privaten Vorschau für Anfragekoordination und Wallet-Übergabe.", "Prüfen Sie lokale Nachrichten, Anfragen, Belege, Ablauf- und Wiederherstellungszustände.", "Das Annehmen erzeugt eine Wallet-Prüfabsicht, führt aber keine Abrechnung oder Änderung aus.", "Messenger ist eine Roadmap-Vorschau für kurzlebige Relays, kein dauerhafter On-Chain-Chat.", "Öffnen, Löschen, Blockieren oder Melden ändert niemals den Wallet-Abrechnungsstatus."],
    contacts: ["{title}: Hilfe zu lokalen Kontaktbezeichnungen, Empfängerkarten und ausdrücklicher Identitätsprüfung.", "Prüfen Sie lokale Kontaktdaten, Ablauf, Widerruf und Nachweise einer Identitätsänderung.", "Eine gespeicherte Bezeichnung beweist weder Identität noch Vertrauen; geänderte Daten müssen geprüft werden.", "Kontakte bleiben lokal und werden nie als Adress- oder Präsenzgraph hochgeladen.", "Das Entfernen eines lokalen Kontakts widerruft keine externen Rechte und ändert keine Wallet-Abrechnung."]
  },
  es: {
    headings: ["Usar esta vista", "Comportamiento local y seguro"],
    wallet: ["{title}: ayuda local para objetos que requieren una revisión explícita de la cartera.", "Revise el motivo, el origen y el estado local indicados antes de realizar cualquier acción.", "Una acción no disponible permanece bloqueada hasta que la cartera nativa indique un paso seguro.", "La política de la cartera nativa toma la decisión final, no esta vista.", "Los secretos y los datos privados de transporte nunca entran en la ayuda."],
    watchers: ["{title}: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.", "Revise datos deterministas de publicación sin cambiar el estado de la red.", "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura.", "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.", "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."],
    explorer: ["{title}: ayuda sobre la vista previa privada de Explorer para identificadores públicos compatibles.", "Use solo identificadores públicos compatibles de puntos de control, lotes, alertas o pruebas.", "Los identificadores desconocidos, privados, mal formados o no disponibles fallan sin consultar la cartera.", "Explorer es una vista previa de la hoja de ruta con datos locales, no un servicio de datos de cartera.", "Los saldos, contactos, mensajes, notas, rutas y secretos locales nunca entran en Explorer."],
    dapps: ["{title}: ayuda sobre la vista previa local y limitada de dApps y su límite de permisos.", "Revise descriptores locales, intenciones limitadas y resultados explícitos.", "Antes de aceptar, revise alcance, usos, caducidad, valor, comisión, divulgación y revocación.", "dApps es una vista previa de la hoja de ruta: no ejecuta código remoto, URL arbitrarias ni firmas genéricas.", "La cartera vuelve a validar cada intención aceptada; esta vista no modifica sus objetos."],
    messenger: ["{title}: ayuda sobre la vista previa privada de coordinación de solicitudes y su entrega a la cartera.", "Revise mensajes, solicitudes, recibos, caducidad y estados de recuperación locales.", "Aceptar crea una intención para revisar en la cartera, pero no liquida ni modifica su estado.", "Messenger es una vista previa de la hoja de ruta para relés breves, no un chat permanente en cadena.", "Abrir, borrar, bloquear o denunciar nunca cambia el estado de liquidación de la cartera."],
    contacts: ["{title}: ayuda sobre etiquetas locales, tarjetas receptoras y revisión explícita de identidad.", "Revise los datos locales, la caducidad, la revocación y las pruebas de cambio de identidad.", "Una etiqueta guardada no demuestra identidad ni confianza; los datos modificados requieren revisión.", "Los contactos permanecen locales y nunca se publican como un grafo de direcciones o presencia.", "Eliminar un contacto local no revoca derechos externos ni cambia la liquidación de la cartera."]
  },
  pt: {
    headings: ["Utilizar esta vista", "Comportamento local e seguro"],
    wallet: ["{title}: ajuda local para objetos que exigem revisão explícita da carteira.", "Reveja o motivo, a origem e o estado local indicados antes de qualquer ação.", "Uma ação indisponível permanece bloqueada até a carteira nativa indicar um passo seguro.", "A política da carteira nativa toma a decisão final, não esta vista.", "Segredos e dados de transporte privados nunca entram na Ajuda."],
    watchers: ["{title}: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.", "Consulte dados determinísticos de publicação sem alterar o estado da rede.", "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança.", "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.", "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."],
    explorer: ["{title}: ajuda sobre a pré-visualização privada do Explorer para identificadores públicos suportados.", "Use apenas identificadores públicos suportados de pontos de controlo, lotes, alertas ou evidência.", "Identificadores desconhecidos, privados, inválidos ou indisponíveis falham sem consultar a carteira.", "Explorer é uma pré-visualização do roteiro com dados locais, não um serviço de dados da carteira.", "Saldos, contactos, mensagens, notas, rotas e segredos locais nunca entram no Explorer."],
    dapps: ["{title}: ajuda sobre a pré-visualização local limitada de dApps e o respetivo limite de permissões.", "Reveja descritores locais, intenções limitadas e resultados explícitos.", "Antes de aceitar, reveja âmbito, utilizações, validade, valor, taxa, divulgação e revogação.", "dApps é uma pré-visualização do roteiro: não executa código remoto, URL arbitrários nem assinatura genérica.", "A carteira revalida cada intenção aceite; esta vista não altera objetos da carteira."],
    messenger: ["{title}: ajuda sobre a pré-visualização privada de coordenação de pedidos e passagem para a carteira.", "Reveja mensagens, pedidos, recibos, validade e estados de recuperação locais.", "Aceitar cria uma intenção para revisão na carteira, mas não liquida nem altera o estado.", "Messenger é uma pré-visualização do roteiro para retransmissão temporária, não conversa permanente na cadeia.", "Abrir, eliminar, bloquear ou denunciar nunca altera o estado de liquidação da carteira."],
    contacts: ["{title}: ajuda sobre etiquetas locais, cartões de receção e revisão explícita da identidade.", "Reveja dados locais, validade, revogação e evidência de alteração da identidade.", "Uma etiqueta guardada não prova identidade nem confiança; dados alterados exigem revisão.", "Os contactos permanecem locais e nunca são publicados como grafo de endereços ou presença.", "Remover um contacto local não revoga direitos externos nem altera a liquidação da carteira."]
  },
  ko: {
    headings: ["이 화면 사용", "로컬 및 안전 동작"],
    wallet: ["{title}: 지갑의 명시적 검토가 필요한 객체에 대한 로컬 도움말입니다.", "작업하기 전에 표시된 이유, 출처, 로컬 상태를 확인하세요.", "네이티브 지갑이 안전한 다음 단계를 제공할 때까지 사용할 수 없는 작업은 차단됩니다.", "최종 결정은 이 화면이 아니라 네이티브 지갑 정책이 내립니다.", "비밀 정보와 비공개 전송 데이터는 도움말에 포함되지 않습니다."],
    watchers: ["{title}: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.", "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.", "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다.", "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.", "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."],
    explorer: ["{title}: 지원되는 공개 식별자만 다루는 개인정보 보호형 Explorer 미리보기 도움말입니다.", "지원되는 공개 검사점, 배치, 알림 또는 증거 식별자만 사용하세요.", "알 수 없거나 비공개이거나 잘못되었거나 사용할 수 없는 식별자는 지갑 조회 없이 차단됩니다.", "Explorer는 로컬 픽스처 기반 로드맵 미리보기이며 지갑 데이터 서비스가 아닙니다.", "로컬 잔액, 연락처, 메시지, 메모, 경로, 비밀 정보는 Explorer에 포함되지 않습니다."],
    dapps: ["{title}: 제한된 로컬 dApps 미리보기와 권한 경계에 대한 도움말입니다.", "로컬 설명자, 범위가 제한된 인텐트, 명시적 결과를 검토하세요.", "수락 전에 범위, 사용 횟수, 만료, 금액, 수수료, 공개, 취소 조건을 확인하세요.", "dApps는 로드맵 미리보기이며 원격 코드, 임의 URL, 범용 서명을 실행하지 않습니다.", "수락한 인텐트는 지갑이 다시 검증하며 이 화면은 지갑 객체를 변경할 수 없습니다."],
    messenger: ["{title}: 비공개 요청 조정 미리보기와 지갑 전달에 대한 도움말입니다.", "로컬 메시지, 요청, 영수증, 만료, 복구 상태를 검토하세요.", "요청 수락은 지갑 검토 인텐트를 만들 뿐 결제하거나 지갑 상태를 변경하지 않습니다.", "Messenger는 단기 릴레이용 로드맵 미리보기이며 영구 온체인 채팅이 아닙니다.", "열기, 삭제, 차단, 신고는 지갑 결제 상태를 변경하지 않습니다."],
    contacts: ["{title}: 로컬 연락처 레이블, 수신자 카드, 명시적 신원 변경 검토에 대한 도움말입니다.", "로컬 연락처 데이터, 만료, 취소, 신원 변경 증거를 검토하세요.", "저장된 레이블은 신원이나 신뢰를 증명하지 않으며 변경된 데이터는 검토해야 합니다.", "연락처는 로컬에 유지되며 주소 또는 온라인 상태 그래프로 업로드되지 않습니다.", "로컬 연락처 삭제는 외부 권한을 취소하거나 지갑 결제를 변경하지 않습니다."]
  },
  tr: {
    headings: ["Bu görünümü kullanma", "Yerel ve güvenli davranış"],
    wallet: ["{title}: açık cüzdan incelemesi gerektiren nesneler için yerel yardım.", "Herhangi bir işlemden önce belirtilen nedeni, kaynağı ve yerel durumu inceleyin.", "Yerel cüzdan güvenli bir sonraki adım sunana kadar kullanılamayan işlem engelli kalır.", "Son kararı bu görünüm değil, yerel cüzdan ilkesi verir.", "Gizli bilgiler ve özel aktarım verileri Yardım’a asla girmez."],
    watchers: ["{title}: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.", "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.", "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur.", "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.", "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."],
    explorer: ["{title}: desteklenen genel kimlikler için gizlilik sınırlı Explorer önizlemesi hakkında yardım.", "Yalnızca desteklenen genel kontrol noktası, parti, uyarı veya kanıt kimliklerini kullanın.", "Bilinmeyen, özel, bozuk veya kullanılamayan kimlikler cüzdan sorgusu olmadan başarısız olur.", "Explorer, yerel verili bir yol haritası önizlemesidir; cüzdan veri hizmeti değildir.", "Yerel bakiyeler, kişiler, mesajlar, notlar, yollar ve gizli bilgiler Explorer’a girmez."],
    dapps: ["{title}: sınırlı yerel dApps önizlemesi ve izin sınırı hakkında yardım.", "Yerel tanımları, kapsamlı niyetleri ve açık sonuçları inceleyin.", "Kabulden önce kapsamı, kullanımları, süreyi, değeri, ücreti, açıklamayı ve iptali inceleyin.", "dApps bir yol haritası önizlemesidir; uzak kod, rastgele URL veya genel imza çalıştırmaz.", "Kabul edilen niyeti Cüzdan yeniden doğrular; bu görünüm cüzdan nesnelerini değiştirmez."],
    messenger: ["{title}: özel istek eşgüdümü önizlemesi ve Cüzdan aktarımı hakkında yardım.", "Yerel mesajları, istekleri, makbuzları, süreyi ve kurtarma durumlarını inceleyin.", "Kabul, Cüzdan inceleme niyeti oluşturur; ödeme yapmaz veya cüzdan durumunu değiştirmez.", "Messenger kısa süreli aktarım için yol haritası önizlemesidir; kalıcı zincir üstü sohbet değildir.", "Açma, silme, engelleme veya bildirme Cüzdan ödeme durumunu değiştirmez."],
    contacts: ["{title}: yerel kişi etiketleri, alıcı kartları ve açık kimlik değişikliği incelemesi hakkında yardım.", "Yerel kişi verilerini, süreyi, iptali ve kimlik değişikliği kanıtını inceleyin.", "Kaydedilmiş etiket kimliği veya güveni kanıtlamaz; değişen veriler açıkça incelenmelidir.", "Kişiler yerel kalır ve adres ya da çevrim içi durum grafiği olarak yüklenmez.", "Yerel kişiyi kaldırmak dış hakları iptal etmez veya Cüzdan ödemesini değiştirmez."]
  },
  ja: {
    headings: ["この画面の使い方", "ローカルで安全な動作"],
    wallet: ["{title}：ウォレットによる明示的な確認が必要な対象のローカルヘルプです。", "操作する前に、表示された理由、提供元、ローカル状態を確認してください。", "ネイティブウォレットが安全な次の手順を示すまで、利用不可の操作はブロックされます。", "最終判断はこの画面ではなく、ネイティブウォレットのポリシーが行います。", "秘密情報と非公開の転送データがヘルプに入ることはありません。"],
    watchers: ["{title}：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。", "ネットワーク状態を変更せず、決定的な公開データを確認します。", "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。", "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。", "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"],
    explorer: ["{title}：対応する公開 ID のみを扱うプライバシー制限付き Explorer プレビューのヘルプです。", "対応する公開チェックポイント、バッチ、アラート、証拠 ID のみを使用してください。", "不明、非公開、不正、利用不可の ID はウォレットを参照せず安全側に失敗します。", "Explorer はローカルデータによるロードマッププレビューであり、ウォレットデータサービスではありません。", "ローカル残高、連絡先、メッセージ、メモ、経路、秘密情報は Explorer に入りません。"],
    dapps: ["{title}：制限されたローカル dApps プレビューと権限境界のヘルプです。", "ローカル記述、範囲限定インテント、明示された結果を確認します。", "承認前に範囲、利用回数、有効期限、金額、手数料、開示、取り消しを確認してください。", "dApps はロードマッププレビューであり、外部コード、任意 URL、汎用署名を実行しません。", "承認したインテントはウォレットが再検証し、この画面はウォレット対象を変更しません。"],
    messenger: ["{title}：非公開リクエスト調整プレビューとウォレット引き渡しのヘルプです。", "ローカルのメッセージ、リクエスト、受領書、期限、復旧状態を確認します。", "承認はウォレット確認用インテントを作るだけで、決済や状態変更は行いません。", "Messenger は短期中継のロードマッププレビューであり、永続的なオンチェーンチャットではありません。", "開く、削除、ブロック、報告の操作がウォレット決済状態を変えることはありません。"],
    contacts: ["{title}：ローカル連絡先ラベル、受取カード、明示的な本人情報変更確認のヘルプです。", "ローカルデータ、有効期限、失効、本人情報変更の証拠を確認します。", "保存したラベルは本人確認や信頼の証明ではなく、変更されたデータは確認が必要です。", "連絡先はローカルに保持され、住所や在席状況のグラフとして送信されません。", "ローカル連絡先の削除は外部権限を失効させず、ウォレット決済も変更しません。"]
  },
  "zh-Hans": {
    headings: ["使用此视图", "本地和安全行为"],
    wallet: ["{title}：针对需要钱包明确审核的对象的本地帮助。", "执行任何操作前，请检查显示的原因、来源和本地状态。", "在原生钱包提供安全的后续步骤前，不可用的操作保持阻止状态。", "最终决定由原生钱包策略作出，而不是此视图。", "机密和私密传输数据绝不会进入帮助。"],
    watchers: ["{title}：只读 Watchers 路线图预览及其公开证据边界的帮助。", "查看确定性的发布数据，而不更改网络状态。", "不可用、过期、格式错误和异常状态均会明确显示并安全失败。", "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。", "钱包标签、交易对手、路径、消息和机密不会暴露。"],
    explorer: ["{title}：面向受支持公开标识符、受隐私约束的 Explorer 预览帮助。", "仅使用受支持的公开检查点、批次、警报或证据标识符。", "未知、私密、格式错误或不可用的标识符不会查询钱包，并会安全失败。", "Explorer 是基于本地样本的路线图预览，并非钱包数据服务。", "本地余额、联系人、消息、备注、路径和机密不会进入 Explorer。"],
    dapps: ["{title}：受限本地 dApps 预览及其权限边界的帮助。", "查看本地描述、范围受限的意图和明确结果。", "接受前请检查范围、使用次数、期限、金额、费用、披露和撤销条件。", "dApps 是路线图预览，不会执行远程代码、任意网址或通用签名。", "钱包会重新验证已接受的意图；此视图不能更改钱包对象。"],
    messenger: ["{title}：私密请求协调预览及其钱包交接的帮助。", "查看本地消息、请求、回执、到期和恢复状态。", "接受请求只会创建钱包审核意图，不会结算或更改钱包状态。", "Messenger 是短期中继的路线图预览，并非永久链上聊天。", "打开、删除、屏蔽或举报内容不会更改钱包结算状态。"],
    contacts: ["{title}：本地联系人标签、接收卡和明确身份变更审核的帮助。", "查看本地联系人数据、到期、撤销和身份变更证据。", "已保存的标签不能证明身份或信任；变更后的数据需要明确审核。", "联系人保留在本地，不会作为地址或在线状态图上传。", "删除本地联系人不会撤销外部权限或更改钱包结算。"]
  }
});

const SUPPLEMENTAL_COPY = Object.freeze({
  ru: {
    notifications: ["{title}: локальные настройки уведомлений, вибрации и мелодии.", "Сначала включите уведомления, затем выберите режим вибрации и мелодию.", "При отключённых уведомлениях зависимые параметры недоступны.", "Демо не запрашивает системные разрешения.", "Готовое приложение должно явно сообщать, если звук или вибрация недоступны."],
    dataStorage: ["{title}: агрегированные локальные показатели без приватных данных.", "Используйте экран для оценки ресурсов без открытия записей кошелька.", "Показанные значения являются детерминированными демо-данными.", "Контакты, сообщения, маршруты, операции и секреты исключены.", "Готовое приложение должно получать только агрегаты через ограниченную нативную возможность."],
    about: ["{title}: версия, назначение и канал обновлений Z00Z.", "Проверьте текущую версию демо для этой сессии.", "JavaScript-демо задаёт UX-цель для Rust и Tauri.", "Демо не скачивает и не устанавливает обновления.", "Готовое приложение должно проверять подписанный манифест выпуска."]
  },
  fr: {
    notifications: ["{title} : préférences locales de notification, vibration et sonnerie.", "Activez les notifications avant de choisir vibration et sonnerie.", "Les choix dépendants sont désactivés lorsque les notifications le sont.", "La démo ne demande aucune autorisation système.", "L’application doit signaler clairement une capacité audio ou haptique indisponible."],
    dataStorage: ["{title} : compteurs locaux agrégés sans données privées.", "Consultez les ressources sans ouvrir les enregistrements du portefeuille.", "Les valeurs affichées sont des données de démonstration déterministes.", "Contacts, messages, routes, activités et secrets sont exclus.", "L’application doit obtenir uniquement des agrégats via une capacité native limitée."],
    about: ["{title} : version, objectif et canal de mise à jour Z00Z.", "Vérifiez la version de démonstration de cette session.", "La démo JavaScript définit la cible UX pour Rust et Tauri.", "La démo ne télécharge ni n’installe de mise à jour.", "L’application doit vérifier un manifeste de version signé."]
  },
  de: {
    notifications: ["{title}: lokale Einstellungen für Benachrichtigung, Vibration und Klingelton.", "Aktivieren Sie Benachrichtigungen, bevor Sie Vibration und Ton wählen.", "Abhängige Optionen sind bei deaktivierten Benachrichtigungen gesperrt.", "Die Demo fordert keine Systemberechtigung an.", "Die App muss fehlende Audio- oder Haptikfunktionen klar melden."],
    dataStorage: ["{title}: aggregierte lokale Zähler ohne private Daten.", "Prüfen Sie Ressourcen, ohne Wallet-Datensätze zu öffnen.", "Die Werte sind deterministische Demodaten.", "Kontakte, Nachrichten, Routen, Aktivitäten und Geheimnisse sind ausgeschlossen.", "Die App darf Aggregate nur über eine begrenzte native Funktion beziehen."],
    about: ["{title}: Z00Z-Version, Zweck und Aktualisierungskanal.", "Prüfen Sie die aktuelle Demoversion dieser Sitzung.", "Die JavaScript-Demo definiert das UX-Ziel für Rust und Tauri.", "Die Demo lädt oder installiert keine Aktualisierung.", "Die App muss ein signiertes Veröffentlichungsmanifest prüfen."]
  },
  es: {
    notifications: ["{title}: preferencias locales de notificación, vibración y tono.", "Active las notificaciones antes de elegir vibración y tono.", "Las opciones dependientes se desactivan al apagar las notificaciones.", "La demo no solicita permisos del sistema.", "La aplicación debe indicar claramente si no hay sonido o vibración."],
    dataStorage: ["{title}: contadores locales agregados sin datos privados.", "Revise recursos sin abrir registros de la cartera.", "Los valores mostrados son datos de demostración deterministas.", "Se excluyen contactos, mensajes, rutas, actividad y secretos.", "La aplicación solo debe obtener agregados mediante una capacidad nativa limitada."],
    about: ["{title}: versión, propósito y canal de actualizaciones de Z00Z.", "Compruebe la versión actual de la demo para esta sesión.", "La demo JavaScript define el objetivo UX para Rust y Tauri.", "La demo no descarga ni instala actualizaciones.", "La aplicación debe verificar un manifiesto de versión firmado."]
  },
  pt: {
    notifications: ["{title}: preferências locais de notificação, vibração e toque.", "Ative as notificações antes de escolher vibração e toque.", "As opções dependentes ficam desativadas quando as notificações estão desligadas.", "A demonstração não pede permissões do sistema.", "A aplicação deve indicar claramente som ou vibração indisponíveis."],
    dataStorage: ["{title}: contadores locais agregados sem dados privados.", "Consulte recursos sem abrir registos da carteira.", "Os valores apresentados são dados de demonstração determinísticos.", "Contactos, mensagens, rotas, atividade e segredos são excluídos.", "A aplicação deve obter apenas agregados por uma capacidade nativa limitada."],
    about: ["{title}: versão, objetivo e canal de atualização Z00Z.", "Verifique a versão atual da demonstração nesta sessão.", "A demonstração JavaScript define o objetivo UX para Rust e Tauri.", "A demonstração não transfere nem instala atualizações.", "A aplicação deve verificar um manifesto de versão assinado."]
  },
  ko: {
    notifications: ["{title}: 알림, 진동 및 벨소리의 로컬 설정입니다.", "진동과 벨소리를 선택하기 전에 알림을 켜세요.", "알림이 꺼지면 관련 선택 항목도 비활성화됩니다.", "데모는 운영 체제 권한을 요청하지 않습니다.", "패키지 앱은 소리나 진동을 사용할 수 없을 때 명확히 알려야 합니다."],
    dataStorage: ["{title}: 비공개 데이터가 없는 집계 로컬 카운터입니다.", "지갑 기록을 열지 않고 리소스 사용량을 확인하세요.", "표시 값은 결정적 데모 데이터입니다.", "연락처, 메시지, 경로, 활동 및 비밀은 제외됩니다.", "패키지 앱은 제한된 네이티브 기능으로 집계 값만 가져와야 합니다."],
    about: ["{title}: Z00Z 버전, 목적 및 업데이트 채널입니다.", "이 세션의 현재 데모 버전을 확인하세요.", "JavaScript 데모는 Rust 및 Tauri UX 목표를 정의합니다.", "데모는 업데이트를 다운로드하거나 설치하지 않습니다.", "패키지 앱은 서명된 릴리스 매니페스트를 확인해야 합니다."]
  },
  tr: {
    notifications: ["{title}: yerel bildirim, titreşim ve zil sesi tercihleri.", "Titreşim ve zil sesi seçmeden önce bildirimleri açın.", "Bildirimler kapalıyken bağlı seçenekler devre dışıdır.", "Demo işletim sistemi izni istemez.", "Paket uygulama ses veya titreşim kullanılamadığında bunu açıkça bildirmelidir."],
    dataStorage: ["{title}: özel veri içermeyen toplu yerel sayaçlar.", "Cüzdan kayıtlarını açmadan kaynak kullanımını inceleyin.", "Gösterilen değerler belirlenimci demo verileridir.", "Kişiler, mesajlar, rotalar, etkinlik ve gizli bilgiler hariç tutulur.", "Paket uygulama yalnızca sınırlı yerel yetenekten toplu değer almalıdır."],
    about: ["{title}: Z00Z sürümü, amacı ve güncelleme kanalı.", "Bu oturumun güncel demo sürümünü kontrol edin.", "JavaScript demosu Rust ve Tauri UX hedefini tanımlar.", "Demo güncelleme indirmez veya kurmaz.", "Paket uygulama imzalı sürüm bildirimini doğrulamalıdır."]
  },
  ja: {
    notifications: ["{title}：通知、振動、着信音のローカル設定です。", "振動と着信音を選ぶ前に通知を有効にします。", "通知が無効な場合、関連する選択肢も無効です。", "デモは OS 権限を要求しません。", "製品版は音声や振動を利用できない場合に明示する必要があります。"],
    dataStorage: ["{title}：非公開データを含まない集計ローカル値です。", "ウォレット記録を開かずにリソース使用量を確認します。", "表示値は決定的なデモデータです。", "連絡先、メッセージ、経路、操作、秘密情報は除外されます。", "製品版は制限されたネイティブ機能から集計値のみを取得する必要があります。"],
    about: ["{title}：Z00Z のバージョン、目的、更新チャネルです。", "このセッションの現在のデモバージョンを確認します。", "JavaScript デモは Rust と Tauri の UX 目標を定義します。", "デモは更新をダウンロードまたはインストールしません。", "製品版は署名済みリリースマニフェストを検証する必要があります。"]
  },
  "zh-Hans": {
    notifications: ["{title}：本地通知、振动和铃声设置。", "选择振动和铃声前，请先启用通知。", "关闭通知时，相关选项也会禁用。", "演示不会请求操作系统权限。", "正式应用必须明确提示声音或振动不可用。"],
    dataStorage: ["{title}：不含私密数据的本地汇总计数。", "无需打开钱包记录即可查看资源使用情况。", "显示值为确定性的演示数据。", "联系人、消息、路径、活动和机密均被排除。", "正式应用只能通过受限原生能力获取汇总值。"],
    about: ["{title}：Z00Z 版本、用途和更新通道。", "检查本次会话的当前演示版本。", "JavaScript 演示定义 Rust 和 Tauri 的 UX 目标。", "演示不会下载或安装更新。", "正式应用必须验证签名的发布清单。"]
  }
});

const NAV_KEY_BY_SEGMENT = Object.freeze({
  assets: "assets",
  vouchers: "vouchers",
  permissions: "permissions",
  quarantine: "quarantine",
  send: "send",
  receive: "receive",
  history: "history",
  swap: "swap",
  exchange: "exchange",
  staking: "staking",
  stake: "stake",
  unstake: "unstake",
  backup: "backup",
  general: "general",
  security: "security",
  policies: "policies",
  advanced: "advanced",
  reticulum: "reticulum",
  onionnet: "onionnet",
  aggregators: "aggregators",
  watchers: "watchers",
  explorer: "explorer",
  overview: "overview",
  node: "node",
  interfaces: "interfaces",
  radio: "radio",
  entrypoints: "entrypoints",
  paths: "paths",
  probes: "probes",
  links: "links",
  epoch: "epoch",
  privacy: "privacy",
  transport: "transport",
  queues: "queues",
  probation: "probation",
  ingress: "ingress",
  planning: "planning",
  placement: "placement",
  publication: "publication",
  recovery: "recovery",
  alerts: "alerts",
  providers: "daProviders",
  censorship: "censorship",
  search: "search",
  checkpoints: "checkpoints",
  batches: "batches",
  discover: "discover",
  installed: "installed",
  connections: "connections",
  activity: "activity",
  inbox: "inbox",
  sent: "sent",
  requests: "requests",
  conversations: "conversations",
  outbox: "outbox",
  receipts: "receipts",
  notifications: "notifications"
});

function titleFor(topicId) {
  const terms = TITLE_TERMS[locale];
  const join = TITLE_JOINERS[locale];
  const parts = topicId.split(".");
  if (topicId === "about") return nav("about");
  if (topicId.startsWith("data-storage.")) return nav(topicId.endsWith("disk-usage") ? "diskUsage" : "networkUsage");
  if (topicId === "asset.details") return `${nav("assets")}${join}${terms.details}`;
  if (topicId === "dapps.detail") return `${nav("dapps")}${join}${terms.details}`;
  if (topicId === "dapps.permission-review") return `${nav("dapps")}${join}${terms.permissionReview}`;
  if (topicId === "messenger.detail") return `${nav("messenger")}${join}${terms.details}`;
  if (topicId === "messenger.request-review") return `${nav("messenger")}${join}${terms.requestReview}`;
  if (topicId === "contacts.detail") return `${nav("contacts")}${join}${terms.details}`;
  if (topicId === "contacts.identity-review") return `${nav("contacts")}${join}${terms.identityReview}`;
  if (topicId === "telemetry.watchers.alert-detail") return `${nav("watchers")}${join}${terms.alertDetails}`;
  if (topicId === "telemetry.explorer.detail") return `${nav("explorer")}${join}${terms.details}`;
  if (parts[0] === "telemetry") {
    const destinationKey = parts[2] === "evidence"
      ? (parts[1] === "watchers" ? "evidenceExport" : "publicEvidence")
      : NAV_KEY_BY_SEGMENT[parts[2]];
    return `${nav(NAV_KEY_BY_SEGMENT[parts[1]])}${join}${nav(destinationKey)}`;
  }
  if (parts[0] === "wallet" && parts[1] === "settings") {
    return `${nav("walletSettings")}${join}${nav(NAV_KEY_BY_SEGMENT[parts[2]])}`;
  }
  if (parts[0] === "contacts") return nav("contacts");
  return nav(NAV_KEY_BY_SEGMENT[parts.at(-1)]);
}

function familyFor(topicId) {
  if (topicId === "about") return "about";
  if (topicId === "settings.notifications") return "notifications";
  if (topicId.startsWith("data-storage.")) return "dataStorage";
  if (topicId.startsWith("wallet.")) return "wallet";
  if (topicId.startsWith("telemetry.watchers.")) return "watchers";
  if (topicId.startsWith("telemetry.explorer.")) return "explorer";
  if (topicId.startsWith("dapps.")) return "dapps";
  if (topicId.startsWith("messenger.")) return "messenger";
  if (topicId.startsWith("contacts.")) return "contacts";
  return "";
}

const familyId = familyFor(payload.topic);
const template = {};
if (familyId) {
  const title = titleFor(payload.topic);
  const copy = COPY[locale];
  const family = (copy[familyId] || SUPPLEMENTAL_COPY[locale][familyId])
    .map((message) => message.replaceAll("{title}", title));
  Object.assign(template, {
    "document.title": title,
    "document.summary": family[0],
    "sections.0.title": copy.headings[0],
    "sections.0.blocks.0.items.0": family[1],
    "sections.0.blocks.0.items.1": family[2],
    "sections.1.title": copy.headings[1],
    "sections.1.blocks.0.items.0": family[3],
    "sections.1.blocks.0.items.1": family[4]
  });
}

const previousMessageHashes = payload.previousMessageHashes
  && typeof payload.previousMessageHashes === "object"
  ? payload.previousMessageHashes
  : null;
const currentMessages = payload.currentMessages && typeof payload.currentMessages === "object"
  ? payload.currentMessages
  : {};
const fallbackKeys = [];
const translated = Object.fromEntries(Object.entries(payload.messages).map(([key, sourceValue]) => {
  const currentValue = currentMessages[key];
  if (payload.topic === "messenger.sent" && typeof template[key] === "string") {
    return [key, template[key]];
  }
  if (
    typeof currentValue === "string"
    && (
      !previousMessageHashes
      || previousMessageHashes[key] === `sha256:${createHash("sha256").update(sourceValue).digest("hex")}`
    )
  ) {
    return [key, currentValue];
  }
  if (typeof currentValue !== "string" && typeof template[key] === "string") {
    return [key, template[key]];
  }
  fallbackKeys.push(key);
  return [key, sourceValue];
}));

process.stdout.write(JSON.stringify({ messages: translated, fallbackKeys }));
