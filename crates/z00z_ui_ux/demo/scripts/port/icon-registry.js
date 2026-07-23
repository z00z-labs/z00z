"use strict";

((root) => {
  const ICON_NAMES = Object.freeze([
    "home", "wallet", "assets", "spark", "activity", "swap", "exchange", "staking",
    "settings", "advanced", "send", "receive", "coin", "token", "nft", "voucher",
    "right", "claim", "permission", "eye", "eye-off", "lock", "shield", "bell", "user",
    "chevron", "copy", "check", "close", "moon", "sun", "more", "alert", "search",
    "backup", "network", "overview", "reticulum-node", "reticulum-interface", "entry",
    "reticulum-paths", "queue", "probe", "reticulum-link", "plus", "logout", "remove"
  ]);

  const OBJECT_FAMILY_ICON_LUT = Object.freeze({
    voucher: Object.freeze({
      iconSrc: "assets/z00z-friendly/Vauchers/vaucher.svg",
      mode: "mask",
      className: "is-voucher"
    }),
    right: Object.freeze({
      iconSrc: "assets/z00z-friendly/Permissions/permission.svg",
      mode: "mask",
      className: "is-right"
    })
  });

  const VOUCHER_ICON_LUT = Object.freeze({
    refund: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-orange.svg", mode: "image", className: "is-voucher" }),
    redeemed: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-green.svg", mode: "image", className: "is-voucher" }),
    travel: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-blue.svg", mode: "image", className: "is-voucher" }),
    gift: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-violet.svg", mode: "image", className: "is-voucher" }),
    service: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-indigo.svg", mode: "image", className: "is-voucher" }),
    deposit: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-yellow.svg", mode: "image", className: "is-voucher" }),
    restricted: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-red.svg", mode: "image", className: "is-voucher" }),
    community: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-white.svg", mode: "image", className: "is-voucher" })
  });

  const PERMISSION_ICON_LUT = Object.freeze({
    receipt: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-blue.svg", mode: "image", className: "is-right" }),
    deploy: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-green.svg", mode: "image", className: "is-right" }),
    publish: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-violet.svg", mode: "image", className: "is-right" }),
    approve: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-yellow.svg", mode: "image", className: "is-right" }),
    audit: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-indigo.svg", mode: "image", className: "is-right" }),
    device: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-orange.svg", mode: "image", className: "is-right" }),
    emergency: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-red.svg", mode: "image", className: "is-right" }),
    view: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-white.svg", mode: "image", className: "is-right" })
  });

  const OBJECT_TYPE_ICON_LUT = Object.freeze({
    asset: Object.freeze({
      coin: Object.freeze({ iconName: "coin", className: "is-coin" }),
      token: Object.freeze({ iconName: "token", className: "is-token" }),
      nft: Object.freeze({ iconName: "nft", className: "is-nft" })
    }),
    voucher: VOUCHER_ICON_LUT,
    right: PERMISSION_ICON_LUT
  });

  Object.assign(root.Z00ZDemo ||= {}, {
    ICON_NAMES,
    OBJECT_FAMILY_ICON_LUT,
    OBJECT_TYPE_ICON_LUT,
    VOUCHER_ICON_LUT,
    PERMISSION_ICON_LUT
  });
})(typeof window === "undefined" ? globalThis : window);
