"use strict";

window.Z00ZI18n.registerLocale("ko", {
  common: { unavailable: "사용할 수 없음", readOnly: "읽기 전용", chain: "체인", chainLocked: "신중하게 선택하세요. 지갑을 만든 후에는 체인을 변경할 수 없습니다.", on: "켬", off: "끔", close: "닫기", back: "뒤로" },
  app: {
    documentTitle: "Z00Z Wallet — 인터랙티브 콘셉트", menu: "메뉴", wallets: "지갑", network: "네트워크", addWallet: "지갑 추가", removeWallet: "지갑 제거", settings: "설정", home: "홈",
    homeContext: "개인 자산을 한눈에", settingsContext: "애플리케이션 환경설정", walletContext: "{wallet} 지갑", interactiveConcept: "인터랙티브 콘셉트 · 실제 자금 없음", conceptBuild: "콘셉트 0.4 · 실제 자금 없음", logOut: "로그아웃",
    general: "일반", language: "언어", languageHelp: "이 지갑 애플리케이션 전체에 적용되는 언어", notifications: "알림", notificationsHelp: "지갑 업데이트와 필요한 작업 표시",
    regionalFormat: "지역 형식", regionalFormatHelp: "언어와 별개로 날짜, 숫자 및 소수점 구분자를 제어합니다", timeZone: "시간대", timeZoneHelp: "타임스탬프는 UTC로 저장되고 이 시간대로 표시됩니다",
    networkUnits: "네트워크 단위", networkUnitsHelp: "네트워크 속도는 초당 십진 비트를 사용합니다", decimalBitrate: "초당 십진 비트", translationCoverage: "번역 적용 범위", translationCoverageHelp: "{count}개 언어 카탈로그가 영어 원본 키와 동기화되었습니다", languageChanged: "언어가 로컬에서 변경되었습니다."
  },
  nav: { home: "홈", assets: "자산", history: "기록", swap: "스왑", exchange: "거래소", staking: "스테이킹", backup: "백업", settings: "설정" },
  network: { routeTelemetry: "경로 텔레메트리", carrierTelemetry: "전송 텔레메트리", publicationTelemetry: "발행 텔레메트리" },
  walletShell: {
    balanceAvailable: "{value} 사용 가능", current: "현재", scanning: "스캔 중", identityAria: "지갑 전환. 현재 지갑: {wallet}", lockLabel: "{wallet} 지갑", copyAddress: "{wallet} 지갑의 전체 주소 복사",
    available: "사용 가능", locked: "잠김", pendingIn: "입금 대기", pendingOut: "출금 대기", routeTelemetry: "경로 텔레메트리"
  },
  assets: {
    wallet: "지갑", sections: "지갑 섹션", sectionAssets: "자산", sectionAssetsHelp: "사용 가능하고 보유한 가치", sectionVouchers: "바우처", sectionVouchersHelp: "조건부 가치", sectionPermissions: "권한", sectionPermissionsHelp: "제한된 권한",
    family: "자산 분류", title: "내 자산", description: "네이티브 화폐, 발행 토큰 및 수집품은 구분되어 유지됩니다. 사용 가능한 네이티브 화폐만 사용 가능에 포함됩니다.", send: "보내기", moneyTotals: "금액 합계", available: "사용 가능", readyToUse: "사용 준비됨", receiving: "수신 중", sending: "전송 중", waitingToSettle: "정산 대기 중",
    filters: "자산 필터", all: "전체", filterCoins: "코인", filterTokens: "토큰", filterNfts: "NFT", kindCoin: "코인", kindToken: "토큰", kindCollectible: "NFT", needsReview: "검토 필요", owned: "보유 자산", ownedHelp: "분류, 신뢰 및 사용 가능 여부가 명시됩니다", name: "이름", balance: "잔액", value: "가치", price: "가격",
    viewDetails: "{asset} 세부정보 보기", receiveAsset: "{asset} 받기", sendAsset: "{asset} 보내기", receive: "받기", noMarketFeed: "시장 피드 없음", nativeCatalog: "네이티브 자산 · 신뢰된 카탈로그", declaredDomain: "선언된 도메인 · 검토 필요", uniqueCollectible: "고유 수집품 · 메타데이터 사용 가능", excludedNotice: "바우처, 권한, 격리된 객체, 비네이티브 토큰, 수집품 및 실험적 호환 자산은 사용 가능에서 제외됩니다.",
    noVouchers: "아직 바우처가 없습니다", noVouchersHelp: "이 지갑에 양도 가능한 조건부 가치가 필요할 때 바우처를 만드세요.", createVoucher: "바우처 만들기", noPermissions: "아직 권한이 없습니다", noPermissionsHelp: "이 지갑에 양도 가능한 권한이 필요할 때 범위가 제한된 권한을 만드세요.", createPermission: "권한 만들기"
  },
  history: {
    honestSettlement: "투명한 정산", title: "변경된 모든 항목", description: "제출과 최종 정산은 별도의 상태로 표시됩니다. 항목을 열어 영수증과 기술 타임라인을 확인하세요.", filters: "기록 필터", all: "전체", assets: "자산", vouchers: "바우처", permissions: "권한", system: "시스템", needsAttention: "주의 필요", search: "기록 검색", results: "기록 결과",
    settling: "정산 중", settled: "정산됨", active: "활성", ready: "준비됨", paymentTo: "{recipient}에게 결제", sentWaiting: "전송됨 · 정산 대기", allocationClaimed: "할당 수령", verifiedClaimWaiting: "수령 확인됨 · 정산 대기", receivedFrom: "{sender}에게서 받음", yesterday: "어제", travelRefundVoucher: "여행 환불 바우처", offeredReviewBefore: "제안됨 · {date} 이전에 검토", deliveryReceiptAccess: "배송 영수증 접근", dataAccessUsesRemain: "데이터 접근 · {total}회 중 {used}회 남음", uses: "{count}회 사용", jul12: "7월 12일", localBackupCreated: "로컬 백업 생성됨", integrityPassed: "무결성 검사 통과", jul10: "7월 10일", transferFrom: "{wallet}에서 전송", jul3: "7월 3일", recoveryCheckCompleted: "복구 검사 완료", localVerificationPassed: "로컬 검증 통과", jun30: "6월 30일", jul21: "7월 21일", waitingToSettle: "정산 대기", minutesAgo: "{count}분 전", noMatching: "일치하는 기록이 없습니다", tryAnother: "다른 필터나 검색어를 사용해 보세요.", details: "기록 세부정보", status: "상태", when: "시각", fee: "수수료", feeIncluded: "포함됨", feeNotApplicable: "해당 없음", privacy: "프라이버시", privacyValue: "대상 시뮬레이션 · 실시간 텔레메트리가 아님", carrierChain: "전송 및 체인", carrierChainValue: "Reticulum 대상 · 메인 모의 환경", technicalDetails: "기술 세부정보", idLabel: "ID", lifecycleLabel: "수명 주기", lifecyclePending: "생성 → 제출 → 승인", lifecycleConfirmed: "생성 → 제출 → 승인 → 확인", receiptLabel: "영수증", copyReceipt: "영수증 복사", done: "완료"
  },
  staking: {
    eyebrow: "지갑 스테이킹", heading: "{wallet}에서 스테이킹", description: "스테이킹 조건과 대기 중인 가치는 지갑의 사용 가능한 잔액과 구분됩니다.", badge: "메인 · 콘셉트", totals: "스테이킹 합계", availableToStake: "스테이킹 가능", walletValueBefore: "스테이킹 제출 전 지갑 가치", staked: "스테이킹됨", nothingDelegated: "이 콘셉트에서는 위임된 항목이 없습니다", rewards: "보상", accrualNotSimulated: "보상 누적은 시뮬레이션하지 않습니다", prepare: "스테이킹 준비", prepareHelp: "승인 전에 금액을 선택하고 검증인 조건을 확인하세요.", amount: "금액", availableBalance: "사용 가능: {value}", validator: "검증인", validatorPlaceholder: "체인 검증 후 선택", review: "스테이킹 검토", safeguards: "스테이킹 보호 장치", validatorStatus: "검증인 상태", unlockPeriod: "잠금 해제 기간", notSelected: "선택되지 않음", notProjected: "예측되지 않음", notice: "이 콘셉트는 수익률을 추정하거나 잠금 위험을 숨기지 않습니다. 스테이킹은 체인 정산까지 대기 상태로 남습니다."
  },
  settings: { sections: "설정 섹션", application: "애플리케이션", connectivity: "연결", general: "일반", generalHelp: "언어 및 애플리케이션 알림", appearance: "모양", appearanceHelp: "테마, 팔레트, 밀도 및 YAML 강조", networkPrivacy: "네트워크", networkPrivacyHelp: "비공개 경로, 체인 및 동기화", networkSections: "네트워크 섹션", overview: "개요" },
  walletSettings: {
    password: "지갑 비밀번호", passwordHelp: "이 지갑 비밀번호를 로컬에서 변경합니다. 새 비밀번호를 적용하려면 현재 비밀번호가 필요합니다.", changePassword: "비밀번호 변경", changePasswordTitle: "지갑 비밀번호 변경", changePasswordSubtitle: "현재 비밀번호를 확인한 후 새 비밀번호를 선택하세요", currentPassword: "현재 비밀번호", newPassword: "새 비밀번호", confirmNewPassword: "새 비밀번호 확인", passwordChangeHint: "8자 이상을 사용하세요. 비밀번호는 데모에서 즉시 지워집니다.", changePasswordSubmit: "비밀번호 변경", passwordChangedTitle: "비밀번호 업데이트됨", passwordChangedResult: "비밀번호는 이 로컬 콘셉트에서만 변경되었습니다. 데모는 모든 입력을 지우며 비밀을 보관하지 않습니다.", passwordCurrentError: "현재 비밀번호를 입력하세요(8자 이상).", passwordNewError: "새 비밀번호는 8자 이상이어야 합니다.", passwordSameError: "현재 비밀번호와 다른 비밀번호를 선택하세요.", passwordMismatchError: "새 비밀번호가 일치하지 않습니다."
  },
  reticulum: { title: "Reticulum 텔레메트리", summary: "전송 및 전달 근거를 보여 주는 로컬 전용 보기입니다. 전송 메타데이터를 사용자 분석으로 바꾸지 않고 가용성 판단을 지원합니다.", localCapability: "로컬 기능을 사용할 수 없음", localCapabilityHelp: "이 지갑 데모에는 등록된 로컬 Reticulum 상태 브리지가 없습니다. 실제처럼 보이는 전송 데이터를 만들지 않습니다.", tabs: { overview: "개요", node: "노드", interfaces: "인터페이스", radio: "라디오", entrypoints: "진입점", paths: "경로", probes: "프로브", links: "링크" } },
  aggregators: { title: "애그리게이터 텔레메트리", summary: "집계 작업을 위한 읽기 전용 서비스 및 발행 근거입니다. 지갑 키, 시드 또는 정책 비밀을 절대 받지 않습니다.", localCapability: "로컬 기능을 사용할 수 없음", localCapabilityHelp: "지갑에 애그리게이터 상태 스냅샷용 등록 브리지가 없으므로 이 페이지는 올바르게 사용할 수 없음을 표시합니다.", tabs: { overview: "개요" } },
  onionnet: { title: "OnionNet 텔레메트리", summary: "결정론적 제어 평면 상태, 로컬 근거 및 집계된 합성 상태를 경계에 맞게 보여 줍니다. 이 작업 공간은 경로나 프라이버시 정책을 변경하지 않습니다.", localCapability: "로컬 기능을 사용할 수 없음", localCapabilityHelp: "이 지갑 데모에는 등록된 OnionNet 상태 브리지가 없습니다. 실제처럼 보이는 경로, 토폴로지 또는 프라이버시 값을 만들지 않습니다.", tabs: { overview: "개요", epoch: "에포크", privacy: "프라이버시", transport: "전송", queues: "큐 및 재생", probation: "검증 기간", ingress: "수신 경계" } },
  status: { up: "정상", down: "중지", connecting: "연결 중", degraded: "성능 저하" },
  units: { bitPerSecond: "{value} bit/s", kilobitPerSecond: "{value} kbit/s", megabitPerSecond: "{value} Mbit/s" }
});
