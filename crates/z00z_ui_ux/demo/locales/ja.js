"use strict";

window.Z00ZI18n.registerLocale("ja", {
  common: { unavailable: "利用不可", readOnly: "読み取り専用", chain: "チェーン", chainLocked: "慎重に選択してください。ウォレットの作成後にチェーンを変更することはできません。", on: "オン", off: "オフ", close: "閉じる" },
  app: {
    documentTitle: "Z00Z Wallet — インタラクティブ概念", wallets: "ウォレット", network: "ネットワーク",
    addWallet: "ウォレットを追加", removeWallet: "ウォレットを削除", settings: "設定", home: "ホーム",
    homeContext: "プライベートな資産をひと目で確認", settingsContext: "アプリケーション設定", walletContext: "{wallet} ウォレット", interactiveConcept: "インタラクティブ概念 · 実際の資金は使用しません", conceptBuild: "概念 0.4 · 実際の資金は使用しません", logOut: "ログアウト",
    general: "一般", language: "言語", languageHelp: "このウォレットアプリケーション全体で使用する言語",
    notifications: "通知", notificationsHelp: "ウォレットの更新と必要な操作を表示します",
    regionalFormat: "地域形式", regionalFormatHelp: "言語とは別に日付、数字、小数点区切りを制御します",
    timeZone: "タイムゾーン", timeZoneHelp: "タイムスタンプは UTC で保存され、このタイムゾーンで表示されます",
    networkUnits: "ネットワーク単位", networkUnitsHelp: "ネットワーク速度には十進ビット毎秒を使用します",
    decimalBitrate: "十進ビット/秒", translationCoverage: "翻訳カバレッジ",
    translationCoverageHelp: "{count} 個の言語カタログが英語のソースキーと同期されています",
    languageChanged: "言語をローカルで変更しました。"
  },
  nav: { home: "ホーム", assets: "資産", history: "履歴", swap: "スワップ", exchange: "取引所", staking: "ステーキング", backup: "バックアップ", settings: "設定" },
  network: { routeTelemetry: "経路テレメトリー", carrierTelemetry: "キャリアテレメトリー", publicationTelemetry: "公開テレメトリー" },
  walletShell: {
    balanceAvailable: "{value} 利用可能",
    current: "現在", scanning: "スキャン中", identityAria: "ウォレットを切り替え。現在のウォレット: {wallet}", lockLabel: "{wallet} ウォレット", copyAddress: "{wallet} ウォレットの完全なアドレスをコピー",
    available: "利用可能", locked: "ロック中", pendingIn: "入金待ち", pendingOut: "送金待ち", routeTelemetry: "経路テレメトリー"
  },
  assets: {
    wallet: "ウォレット", sections: "ウォレットの項目", sectionAssets: "資産", sectionAssetsHelp: "利用可能かつ保有中の価値", sectionVouchers: "バウチャー", sectionVouchersHelp: "条件付きの価値", sectionPermissions: "権限", sectionPermissionsHelp: "制限付きの権限",
    family: "資産区分", title: "あなたの資産", description: "ネイティブ通貨、発行トークン、コレクティブルは区別されます。利用可能には支出可能なネイティブ通貨のみが含まれます。", send: "送信", moneyTotals: "資金合計", available: "利用可能", readyToUse: "利用準備完了", receiving: "受信中", sending: "送信中", waitingToSettle: "決済待ち",
    filters: "資産フィルター", all: "すべて", filterCoins: "コイン", filterTokens: "トークン", filterNfts: "NFT", kindCoin: "コイン", kindToken: "トークン", kindCollectible: "NFT", needsReview: "要確認", owned: "保有資産", ownedHelp: "区分、信頼性、支出可能性を明示します", name: "名称", balance: "残高", value: "価値", price: "価格",
    viewDetails: "{asset} の詳細を表示", receiveAsset: "{asset} を受信", sendAsset: "{asset} を送信", receive: "受信", noMarketFeed: "市場フィードなし", nativeCatalog: "ネイティブ資産 · 信頼済みカタログ", declaredDomain: "申告済みドメイン · 要確認", uniqueCollectible: "固有のコレクティブル · メタデータ利用可", excludedNotice: "バウチャー、権限、隔離されたオブジェクト、非ネイティブトークン、コレクティブル、実験的な互換資産は利用可能から除外されます。",
    noVouchers: "バウチャーはまだありません", noVouchersHelp: "このウォレットに譲渡可能な条件付き価値が必要な場合は、バウチャーを作成します。", createVoucher: "バウチャーを作成", noPermissions: "権限はまだありません", noPermissionsHelp: "このウォレットに譲渡可能な権限が必要な場合は、範囲を限定した権限を作成します。", createPermission: "権限を作成"
  },
  history: {
    honestSettlement: "正直な決済", title: "変更されたすべて", description: "送信と最終決済は別の状態として表示されます。項目を開くと証跡と技術タイムラインを確認できます。", filters: "履歴フィルター", all: "すべて", assets: "資産", vouchers: "バウチャー", permissions: "権限", system: "システム", needsAttention: "要確認", search: "履歴を検索", results: "履歴の結果",
    settling: "決済中", settled: "決済済み", active: "有効", ready: "準備完了", paymentTo: "{recipient} への支払い", sentWaiting: "送信済み · 決済待ち", allocationClaimed: "配分を請求", verifiedClaimWaiting: "請求を確認 · 決済待ち", receivedFrom: "{sender} から受信", yesterday: "昨日", travelRefundVoucher: "旅行返金バウチャー", offeredReviewBefore: "提示済み · {date} 前に確認", deliveryReceiptAccess: "配達受領書へのアクセス", dataAccessUsesRemain: "データアクセス · 残り {total} 回中 {used} 回", uses: "{count} 回使用", jul12: "7月12日", localBackupCreated: "ローカルバックアップを作成", integrityPassed: "整合性チェックに合格", jul10: "7月10日", transferFrom: "{wallet} からの送金", jul3: "7月3日", recoveryCheckCompleted: "リカバリーチェック完了", localVerificationPassed: "ローカル検証に合格", jun30: "6月30日", jul21: "7月21日", waitingToSettle: "決済待ち", minutesAgo: "{count}分前", noMatching: "一致する履歴はありません", tryAnother: "別のフィルターまたは検索語を試してください。", details: "履歴の詳細", status: "状態", when: "日時", fee: "手数料", feeIncluded: "含まれています", feeNotApplicable: "該当なし", privacy: "プライバシー", privacyValue: "ターゲットのシミュレーション · ライブテレメトリーではありません", carrierChain: "キャリアとチェーン", carrierChainValue: "Reticulum ターゲット · メインネットのモック", technicalDetails: "技術的詳細", idLabel: "ID", lifecycleLabel: "ライフサイクル", lifecyclePending: "作成 → 送信 → 受理", lifecycleConfirmed: "作成 → 送信 → 受理 → 確認", receiptLabel: "受領書", copyReceipt: "受領書をコピー", done: "完了"
  },
  staking: {
    eyebrow: "ウォレットのステーキング", heading: "{wallet} からステーキング", description: "ステーキング条件と保留中の価値は、ウォレットの支出可能な利用可能残高と区別されます。", badge: "メインネット · 概念", totals: "ステーキング合計", availableToStake: "ステーキング可能", walletValueBefore: "ステーキング送信前のウォレット価値", staked: "ステーキング済み", nothingDelegated: "この概念では何も委任されていません", rewards: "報酬", accrualNotSimulated: "報酬の蓄積はシミュレートされません", prepare: "ステーキングを準備", prepareHelp: "認可前に金額を選び、バリデーター条件を確認します。", amount: "金額", availableBalance: "利用可能: {value}", validator: "バリデーター", validatorPlaceholder: "チェーン検証後に選択", review: "ステーキングを確認", safeguards: "ステーキングの保護", validatorStatus: "バリデーターの状態", unlockPeriod: "ロック解除期間", notSelected: "未選択", notProjected: "予測なし", notice: "この概念は利回りを推定せず、ロックアップのリスクも隠しません。ステーキングはチェーン決済まで保留のままです。"
  },
  settings: { sections: "設定セクション", application: "アプリケーション", connectivity: "接続", general: "一般", generalHelp: "言語とアプリケーションの通知", appearance: "外観", appearanceHelp: "テーマ、パレット、密度、YAML ハイライト", networkPrivacy: "ネットワーク", networkPrivacyHelp: "プライベート経路、チェーン、同期", networkSections: "ネットワークセクション", overview: "概要" },
  walletSettings: { password: "ウォレットのパスワード", passwordHelp: "このウォレットのパスワードをローカルで変更します。新しいパスワードを設定する前に現在のパスワードが必要です。", changePassword: "パスワードを変更", changePasswordTitle: "ウォレットのパスワードを変更", changePasswordSubtitle: "現在のパスワードを確認してから新しいパスワードを選択します", currentPassword: "現在のパスワード", newPassword: "新しいパスワード", confirmNewPassword: "新しいパスワードを確認", passwordChangeHint: "8 文字以上を使用してください。パスワードはデモから直ちに消去されます。", changePasswordSubmit: "パスワードを変更", passwordChangedTitle: "パスワードを更新しました", passwordChangedResult: "パスワードはこのローカル概念内でのみ変更されました。デモはすべての入力を消去し、秘密を保持しません。", passwordCurrentError: "現在のパスワードを入力してください（8 文字以上）。", passwordNewError: "新しいパスワードには 8 文字以上を使用してください。", passwordSameError: "現在のパスワードとは異なるパスワードを選択してください。", passwordMismatchError: "新しいパスワードが一致しません。" },
  reticulum: {
    title: "Reticulum テレメトリー",
    summary: "キャリアと配信証拠のローカル専用ビューです。転送メタデータを利用者分析に変えずに可用性判断を支援します。",
    localCapability: "ローカル機能を利用できません",
    localCapabilityHelp: "このウォレットデモにはローカル Reticulum ステータスブリッジが登録されていません。本物らしいキャリアデータは作成しません。",
    tabs: { overview: "概要", node: "ノード", interfaces: "インターフェース", radio: "無線", entrypoints: "エントリポイント", paths: "経路", probes: "プローブ", links: "リンク" }
  },
  aggregators: {
    title: "アグリゲーターのテレメトリー",
    summary: "集約作業のためのサービスおよび公開証拠を読み取り専用で表示します。この画面がウォレットの鍵、シードフレーズ、ポリシー秘密を受け取ることはありません。",
    localCapability: "ローカル機能を利用できません",
    localCapabilityHelp: "ウォレットにはアグリゲーターの状態スナップショットへのブリッジが登録されていないため、このページは正しく利用不可を表示します。",
    tabs: { overview: "概要" }
  },
  onionnet: {
    title: "OnionNet テレメトリー",
    summary: "決定論的なコントロールプレーン状態、ローカル証拠、集約された合成ヘルスをデータ境界に沿って表示します。この画面は経路やプライバシーポリシーを変更しません。",
    localCapability: "ローカル機能を利用できません",
    localCapabilityHelp: "このウォレットデモには OnionNet ステータスブリッジが登録されていません。本物らしい経路、トポロジー、プライバシー値は作成しません。",
    tabs: { overview: "概要", epoch: "エポック", privacy: "プライバシー下限", transport: "転送", queues: "キューと再生", probation: "試用", ingress: "入口境界" }
  },
  status: { up: "稼働中", down: "停止", connecting: "接続中", degraded: "低下" },
  units: { bitPerSecond: "{value} bit/s", kilobitPerSecond: "{value} kbit/s", megabitPerSecond: "{value} Mbit/s" }
});
