"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT) {
    throw new Error("Z00Z demo contracts must load before the Exchange catalogue.");
  }

  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };

  const EXCHANGE_PROVIDER_LUT = deepFreeze({
    hyperliquid: {
      id: "hyperliquid",
      labelKey: "exchange.providerHyperliquid",
      executionKey: "exchange.executionOrderBook",
      iconName: "exchange",
      defaultDestination: "hyperliquid-usdc",
      orderTypes: ["market", "limit"],
      destinationIds: ["hyperliquid-usdc", "hyperliquid-hype", "hyperliquid-btc"]
    },
    "near-intents": {
      id: "near-intents",
      labelKey: "exchange.providerNearIntents",
      executionKey: "exchange.executionSolver",
      iconName: "swap",
      defaultDestination: "solana-usdc",
      orderTypes: [],
      destinationIds: ["solana-usdc", "ethereum-eth", "solana-sol", "near-near", "arbitrum-usdc"]
    }
  });

  const EXCHANGE_DESTINATION_LUT = deepFreeze({
    "hyperliquid-usdc": {
      id: "hyperliquid-usdc",
      providerId: "hyperliquid",
      label: "USDC",
      unit: "USDC",
      network: "Hyperliquid",
      assetId: "hyperliquid:spot:USDC"
    },
    "hyperliquid-hype": {
      id: "hyperliquid-hype",
      providerId: "hyperliquid",
      label: "HYPE",
      unit: "HYPE",
      network: "Hyperliquid",
      assetId: "hyperliquid:spot:HYPE"
    },
    "hyperliquid-btc": {
      id: "hyperliquid-btc",
      providerId: "hyperliquid",
      label: "BTC",
      unit: "BTC",
      network: "Hyperliquid",
      assetId: "hyperliquid:spot:BTC"
    },
    "solana-usdc": {
      id: "solana-usdc",
      providerId: "near-intents",
      label: "USDC",
      unit: "USDC",
      network: "Solana",
      assetId: "nep141:sol-usdc.omft.near"
    },
    "ethereum-eth": {
      id: "ethereum-eth",
      providerId: "near-intents",
      label: "ETH",
      unit: "ETH",
      network: "Ethereum",
      assetId: "nep141:eth.omft.near"
    },
    "solana-sol": {
      id: "solana-sol",
      providerId: "near-intents",
      label: "SOL",
      unit: "SOL",
      network: "Solana",
      assetId: "nep141:sol.omft.near"
    },
    "near-near": {
      id: "near-near",
      providerId: "near-intents",
      label: "NEAR",
      unit: "NEAR",
      network: "NEAR",
      assetId: "nep141:wrap.near"
    },
    "arbitrum-usdc": {
      id: "arbitrum-usdc",
      providerId: "near-intents",
      label: "USDC",
      unit: "USDC",
      network: "Arbitrum",
      assetId: "nep141:arb-usdc.omft.near"
    }
  });

  function exchangeProvider(providerId) {
    return EXCHANGE_PROVIDER_LUT[providerId] || EXCHANGE_PROVIDER_LUT["near-intents"];
  }

  function exchangeDestinations(providerId) {
    const provider = exchangeProvider(providerId);
    return provider.destinationIds.map((id) => EXCHANGE_DESTINATION_LUT[id]).filter(Boolean);
  }

  Object.assign(root.Z00ZDemo, {
    EXCHANGE_PROVIDER_LUT,
    EXCHANGE_DESTINATION_LUT,
    exchangeProvider,
    exchangeDestinations
  });
})(typeof window === "undefined" ? globalThis : window);
