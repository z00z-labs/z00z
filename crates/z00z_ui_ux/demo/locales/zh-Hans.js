"use strict";

window.Z00ZI18n.registerLocale("zh-Hans", {
  common: { unavailable: "不可用", readOnly: "只读", chain: "链", chainLocked: "请谨慎选择。钱包创建后无法更改链。", on: "开", off: "关", close: "关闭" },
  app: {
    documentTitle: "Z00Z Wallet — 交互式概念", wallets: "钱包", network: "网络",
    addWallet: "添加钱包", removeWallet: "移除钱包", settings: "设置", home: "主页",
    homeContext: "一览您的私密资产", settingsContext: "应用偏好设置", walletContext: "{wallet} 钱包", interactiveConcept: "交互式概念 · 不使用真实资金", conceptBuild: "概念 0.4 · 不使用真实资金", logOut: "退出登录",
    general: "常规", language: "语言", languageHelp: "整个钱包应用程序使用的语言",
    notifications: "通知", notificationsHelp: "显示钱包更新和需要处理的操作",
    regionalFormat: "区域格式", regionalFormatHelp: "独立于语言控制日期、数字和小数分隔符",
    timeZone: "时区", timeZoneHelp: "时间戳以 UTC 存储，并按此时区显示",
    networkUnits: "网络单位", networkUnitsHelp: "网络速率使用十进制每秒比特数",
    decimalBitrate: "十进制位/秒", translationCoverage: "翻译覆盖率",
    translationCoverageHelp: "{count} 个语言目录已与英文源键同步",
    languageChanged: "语言已在本地更改。"
  },
  nav: { home: "主页", assets: "资产", history: "历史记录", swap: "兑换", exchange: "交易所", staking: "质押", backup: "备份", settings: "设置" },
  network: { routeTelemetry: "路由遥测", carrierTelemetry: "载体遥测", publicationTelemetry: "发布遥测" },
  walletShell: {
    balanceAvailable: "{value} 可用",
    current: "当前", scanning: "扫描中", identityAria: "切换钱包。当前钱包：{wallet}", lockLabel: "{wallet} 钱包", copyAddress: "复制 {wallet} 钱包的完整地址",
    available: "可用", locked: "已锁定", pendingIn: "待入账", pendingOut: "待转出", routeTelemetry: "路由遥测"
  },
  assets: {
    wallet: "钱包", sections: "钱包分区", sectionAssets: "资产", sectionAssetsHelp: "可用和已持有的价值", sectionVouchers: "凭证", sectionVouchersHelp: "条件价值", sectionPermissions: "权限", sectionPermissionsHelp: "受限授权",
    family: "资产类别", title: "你的资产", description: "原生现金、发行代币和收藏品保持明确区分。只有可花费的原生现金计入可用。", send: "发送", moneyTotals: "资金汇总", available: "可用", readyToUse: "可立即使用", receiving: "接收中", sending: "发送中", waitingToSettle: "等待结算",
    filters: "资产筛选", all: "全部", filterCoins: "币", filterTokens: "代币", filterNfts: "NFT", kindCoin: "代币", kindToken: "通证", kindCollectible: "NFT", needsReview: "需要审核", owned: "已持有资产", ownedHelp: "类别、信任和可花费性均清晰标明", name: "名称", balance: "余额", value: "价值", price: "价格",
    viewDetails: "查看 {asset} 的详情", receiveAsset: "接收 {asset}", sendAsset: "发送 {asset}", receive: "接收", noMarketFeed: "无市场数据", nativeCatalog: "原生资产 · 可信目录", declaredDomain: "已声明域名 · 需要审核", uniqueCollectible: "唯一收藏品 · 元数据可用", excludedNotice: "凭证、权限、隔离对象、非原生通证、收藏品和实验性兼容资产不计入可用。",
    noVouchers: "还没有凭证", noVouchersHelp: "当此钱包需要可转让的条件价值时创建凭证。", createVoucher: "创建凭证", noPermissions: "还没有权限", noPermissionsHelp: "当此钱包需要可转让的权限时创建受限权限。", createPermission: "创建权限"
  },
  history: {
    honestSettlement: "透明结算", title: "所有变更", description: "提交和最终结算会显示为不同状态。打开条目可查看其收据和技术时间线。", filters: "历史筛选", all: "全部", assets: "资产", vouchers: "凭证", permissions: "权限", system: "系统", needsAttention: "需要关注", search: "搜索历史", results: "历史结果",
    settling: "结算中", settled: "已结算", active: "活跃", ready: "就绪", paymentTo: "向 {recipient} 付款", sentWaiting: "已发送 · 等待结算", allocationClaimed: "已领取分配", verifiedClaimWaiting: "领取已验证 · 等待结算", receivedFrom: "从 {sender} 收到", yesterday: "昨天", travelRefundVoucher: "旅行退款凭证", offeredReviewBefore: "已提供 · 请在 {date} 前审核", deliveryReceiptAccess: "送达回执访问权限", dataAccessUsesRemain: "数据访问 · 还剩 {total} 次中的 {used} 次使用", uses: "{count} 次使用", jul12: "7月12日", localBackupCreated: "已创建本地备份", integrityPassed: "完整性检查通过", jul10: "7月10日", transferFrom: "从 {wallet} 转入", jul3: "7月3日", recoveryCheckCompleted: "恢复检查已完成", localVerificationPassed: "本地验证已通过", jun30: "6月30日", jul21: "7月21日", waitingToSettle: "等待结算", minutesAgo: "{count} 分钟前", noMatching: "没有匹配的历史记录", tryAnother: "请尝试其他筛选条件或搜索词。", details: "历史详情", status: "状态", when: "时间", fee: "费用", feeIncluded: "已包含", feeNotApplicable: "不适用", privacy: "隐私", privacyValue: "目标模拟 · 非实时遥测", carrierChain: "传输与链", carrierChainValue: "Reticulum 目标 · 主网模拟", technicalDetails: "技术详情", idLabel: "ID", lifecycleLabel: "生命周期", lifecyclePending: "已创建 → 已提交 → 已接纳", lifecycleConfirmed: "已创建 → 已提交 → 已接纳 → 已确认", receiptLabel: "收据", copyReceipt: "复制收据", done: "完成"
  },
  staking: {
    eyebrow: "钱包质押", heading: "从 {wallet} 质押", description: "质押条款和待结算价值与该钱包可花费的可用余额保持区分。", badge: "主网 · 概念", totals: "质押汇总", availableToStake: "可质押", walletValueBefore: "提交质押前的钱包价值", staked: "已质押", nothingDelegated: "此概念中未委托任何价值", rewards: "奖励", accrualNotSimulated: "不模拟收益累积", prepare: "准备质押", prepareHelp: "在授权前选择金额并查看验证者条款。", amount: "金额", availableBalance: "可用：{value}", validator: "验证者", validatorPlaceholder: "链验证后选择", review: "审核质押", safeguards: "质押保障", validatorStatus: "验证者状态", unlockPeriod: "解锁期", notSelected: "未选择", notProjected: "未预测", notice: "此概念从不估算收益，也不隐藏锁定风险。质押会一直处于待处理状态，直到链上结算。"
  },
  settings: { sections: "设置部分", application: "应用程序", connectivity: "连接", general: "常规", generalHelp: "语言和应用程序通知", appearance: "外观", appearanceHelp: "主题、调色板、密度和 YAML 高亮", networkPrivacy: "网络", networkPrivacyHelp: "私有路由、链和同步", networkSections: "网络部分", overview: "概览" },
  walletSettings: { password: "钱包密码", passwordHelp: "在本地更改此钱包的密码。接受新密码前需要输入当前密码。", changePassword: "更改密码", changePasswordTitle: "更改钱包密码", changePasswordSubtitle: "验证当前密码，然后选择一个新密码", currentPassword: "当前密码", newPassword: "新密码", confirmNewPassword: "确认新密码", passwordChangeHint: "请使用至少 8 个字符。密码会立即从演示中清除。", changePasswordSubmit: "更改密码", passwordChangedTitle: "密码已更新", passwordChangedResult: "密码仅在此本地概念中更改。演示会清除每项输入，不会保留任何秘密。", passwordCurrentError: "请输入当前密码（至少 8 个字符）。", passwordNewError: "新密码请使用至少 8 个字符。", passwordSameError: "请选择与当前密码不同的密码。", passwordMismatchError: "新密码不匹配。" },
  reticulum: {
    title: "Reticulum 遥测",
    summary: "仅在本地查看载体和交付证据。它支持可用性决策，而不会将传输元数据变成用户分析。",
    localCapability: "本地功能不可用",
    localCapabilityHelp: "此钱包演示未注册本地 Reticulum 状态桥接。不会伪造看似实时的载体数据。",
    tabs: { overview: "概览", node: "节点", interfaces: "接口", radio: "无线", entrypoints: "接入点", paths: "路径", probes: "探测", links: "链路" }
  },
  aggregators: {
    title: "聚合器遥测",
    summary: "聚合工作中的服务和发布证据仅供读取。该工作区绝不会接收钱包密钥、助记词或策略机密。",
    localCapability: "本地功能不可用",
    localCapabilityHelp: "钱包尚未注册聚合器状态快照桥接，因此此页面会正确显示不可用。",
    tabs: { overview: "概览" }
  },
  onionnet: {
    title: "OnionNet 遥测",
    summary: "按数据边界展示确定性控制平面状态、本地证据和汇总合成健康信息。此工作区绝不会更改路由或隐私策略。",
    localCapability: "本地功能不可用",
    localCapabilityHelp: "此钱包演示未注册 OnionNet 状态桥接。不会伪造看似实时的路由、拓扑或隐私数值。",
    tabs: { overview: "概览", epoch: "纪元", privacy: "隐私下限", transport: "传输", queues: "队列与重放", probation: "观察期", ingress: "入口边界" }
  },
  status: { up: "运行中", down: "离线", connecting: "正在连接", degraded: "受限" },
  units: { bitPerSecond: "{value} bit/s", kilobitPerSecond: "{value} kbit/s", megabitPerSecond: "{value} Mbit/s" }
});
