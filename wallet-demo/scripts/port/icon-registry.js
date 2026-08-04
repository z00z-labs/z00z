"use strict";

((root) => {
  const demo = root.Z00ZDemo ||= {};
  if (!demo.ICON_NAMES) {
    throw new Error("The canonical icon sprite must load before the object icon registry.");
  }

  const OBJECT_FAMILY_ICON_LUT = Object.freeze({
    voucher: Object.freeze({
      iconSrc: "assets/z00z-friendly/Vauchers/vaucher.svg?v=c9778593beb14fabebcae28a55fc3071e797508a",
      mode: "mask",
      className: "is-voucher"
    }),
    right: Object.freeze({
      iconSrc: "assets/z00z-friendly/Permissions/permission.svg?v=c9778593beb14fabebcae28a55fc3071e797508a",
      mode: "mask",
      className: "is-right"
    })
  });

  const VOUCHER_ICON_LUT = Object.freeze({
    refund: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-orange.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" }),
    redeemed: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-green.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" }),
    travel: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-blue.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" }),
    gift: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-violet.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" }),
    service: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-indigo.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" }),
    deposit: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-yellow.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" }),
    restricted: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-red.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" }),
    community: Object.freeze({ iconSrc: "assets/z00z-friendly/Vauchers/vaucher-white.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-voucher" })
  });

  const PERMISSION_ICON_LUT = Object.freeze({
    receipt: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-blue.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" }),
    deploy: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-green.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" }),
    publish: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-violet.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" }),
    approve: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-yellow.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" }),
    audit: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-indigo.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" }),
    device: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-orange.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" }),
    emergency: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-red.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" }),
    view: Object.freeze({ iconSrc: "assets/z00z-friendly/Permissions/permission-white.svg?v=c9778593beb14fabebcae28a55fc3071e797508a", mode: "image", className: "is-right" })
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

  Object.assign(demo, {
    OBJECT_FAMILY_ICON_LUT,
    OBJECT_TYPE_ICON_LUT,
    VOUCHER_ICON_LUT,
    PERMISSION_ICON_LUT
  });
})(typeof window === "undefined" ? globalThis : window);
