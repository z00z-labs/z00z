"use strict";

((root) => {
  const ICON_SPRITE_MARKUP = String.raw`

      <symbol id="i-home" viewBox="0 0 24 24"><path d="m3 11 9-8 9 8v9a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1z"/></symbol>
      <symbol id="i-menu" viewBox="0 0 24 24"><path d="M4 7h16M4 12h16M4 17h16"/></symbol>
      <symbol id="i-wallet" viewBox="0 0 24 24"><path d="M4 5h14a2 2 0 0 1 2 2v12H4a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2Z"/><path d="M16 10h6v5h-6a2.5 2.5 0 0 1 0-5Z"/><path d="M5 5V3h12"/></symbol>
      <symbol id="i-assets" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M.75 12a11.25 11.25 0 1 0 22.5 0a11.25 11.25 0 0 0-22.5 0"/><path d="M15.187 13.25L10.5 7L6 17"/><path d="M8.813 10.75L13.5 17L18 7"/></g></symbol>
      <symbol id="i-spark" viewBox="0 0 24 24"><path d="m12 3 1.6 4.4L18 9l-4.4 1.6L12 15l-1.6-4.4L6 9l4.4-1.6Z"/><path d="m19 15 .8 2.2L22 18l-2.2.8L19 21l-.8-2.2L16 18l2.2-.8Z"/></symbol>
      <symbol id="i-activity" viewBox="0 0 24 24"><path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5M12 7v5l3 2"/></symbol>
      <symbol id="i-swap" viewBox="0 0 24 24"><path d="M7 7h12m0 0-3-3m3 3-3 3M17 17H5m0 0 3 3m-3-3 3-3"/></symbol>
      <symbol id="i-earn" viewBox="0 0 24 24" data-iconify="material-symbols-light:money-bag-outline" data-menu-icon-mode="outline" data-menu-icon-weight="1.5"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M8.6 20h6.8a4.6 4.6 0 0 0 3.5-7.6L15.4 8H8.6l-3.5 4.4A4.6 4.6 0 0 0 8.6 20Z"/><path d="M9.5 8 7.6 4h8.8l-1.9 4M9.5 12h5M12 10.5v5"/></g></symbol>
      <symbol id="i-dapp-pay" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><rect x="2.75" y="5" width="18.5" height="14" rx="2.5"/><path d="M3 9.25h18M6 15.25h4M13 15.25h3"/></g></symbol>
      <symbol id="i-dapp-request" viewBox="0 0 24 24" data-iconify="mdi-light:pin" data-menu-icon-mode="normalized-fill" data-menu-icon-base-weight="1" data-menu-icon-weight="1.5"><path fill="currentColor" stroke="currentColor" stroke-width="0.5" stroke-linejoin="round" d="M14 12.41V5h1V4H8v1h1v7.41l-2 2V15h9v-.59zM17 14v2h-5v4.5l-.5 1.5l-.5-1.5V16H6v-2l2-2V6H7V3h9v3h-1v6z"/></symbol>
      <symbol id="i-voucher-list" viewBox="0 0 24 24" data-menu-icon-source="wallet-assets:voucher-list" data-menu-icon-mode="source-fill" data-menu-icon-weight="1.5"><path class="icon-fill" d="M5 14h14v1H5zm16 3V8H3v9zM1 5h22v14H1zm4 5h7v2H5z"/></symbol>
      <symbol id="i-permission-list" viewBox="0 0 24 24" data-menu-icon-source="wallet-assets:permission-list" data-menu-icon-mode="outline" data-menu-icon-weight="1.8"><g transform="scale(0.5)" fill="none" stroke="currentColor" stroke-linecap="round" stroke-width="3.6"><path stroke-linejoin="round" d="M20 10H6a2 2 0 0 0-2 2v26a2 2 0 0 0 2 2h36a2 2 0 0 0 2-2v-2.5"/><path d="M10 23h8m-8 8h24"/><circle cx="34" cy="16" r="6" stroke-linejoin="round"/><path stroke-linejoin="round" d="M44 28.419C42.047 24.602 38 22 34 22s-5.993 1.133-8.05 3"/></g></symbol>
      <symbol id="i-dapp-agents-budget" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><rect x="4" y="7" width="16" height="13" rx="3"/><path d="M12 7V4M10 4h4M4 13H2M22 13h-2M8 12h.01M16 12h.01M8 16h8"/></g></symbol>
      <symbol id="i-dapp-wbold-gateway" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="m3 8 9-5 9 5H3ZM5 9v8M9.5 9v8M14.5 9v8M19 9v8M3 18h18M2 21h20"/><path d="m8 13 2-2 2 2M16 13l-2 2-2-2"/></g></symbol>
      <symbol id="i-dapp-subscription" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M6 3v3M18 3v3M4 8h16M5 5h14a1 1 0 0 1 1 1v6M12 21H5a1 1 0 0 1-1-1V6"/><path d="M15 15a4 4 0 0 1 6 1l1-1M22 15v3h-3M21 21a4 4 0 0 1-6-1l-1 1M14 21v-3h3"/></g></symbol>
      <symbol id="i-dapp-donation" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M15.5 3.5c1.6-1.7 5-.6 5 2.2 0 2-2.1 4-5 6.4-2.9-2.4-5-4.4-5-6.4 0-2.8 3.4-3.9 5-2.2Z"/><path d="M3 12h4l5 2h3a2 2 0 0 1 2 2h2a2 2 0 0 1 2 2v1l-8 2-6-2v2H3ZM7 14v5M10 16h7"/></g></symbol>
      <symbol id="i-dapp-escrow" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="12" cy="12" r="4"/><circle cx="12" cy="12" r="1"/><path d="M12 8V6M12 18v-2M8 12H6M18 12h-2M7 21v1M17 21v1"/></g></symbol>
      <symbol id="i-dapp-bounties" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><circle cx="11" cy="13" r="8"/><circle cx="11" cy="13" r="4"/><circle cx="11" cy="13" r="1"/><path d="m13 11 7-7M16 4h4v4"/></g></symbol>
      <symbol id="i-dapp-tickets-passes" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M3 5h18v5a2.5 2.5 0 0 0 0 5v4H3v-4a2.5 2.5 0 0 0 0-5Z"/><path d="M12 7v2M12 11v2M12 15v2"/></g></symbol>
      <symbol id="i-dapp-service-credits" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><rect x="2.5" y="4" width="19" height="15" rx="2"/><path d="M6 8h6M6 12h4M6 16h7"/><circle cx="17" cy="11" r="2.5"/><path d="M15.5 13v4l1.5-1 1.5 1v-4"/></g></symbol>
      <symbol id="i-dapp-digital-goods" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="m12 2.75 9 5v8.5l-9 5-9-5v-8.5Z"/><path d="m3 7.75 9 5 9-5M12 12.75v8.5M7.5 5.25l9 5"/></g></symbol>
      <symbol id="i-dapp-payroll" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><circle cx="7" cy="7" r="3"/><path d="M2 20v-2a5 5 0 0 1 9-3M13 5h9v15h-9zM16 9h3M16 13h3M16 17h2"/></g></symbol>
      <symbol id="i-dapp-private-contract" viewBox="0 0 24 24" data-iconify="et:document" data-menu-icon-mode="source-fill" data-menu-icon-weight="1.5"><g transform="translate(3 0) scale(0.75)" fill="currentColor"><path d="M1.5 32h21c.827 0 1.5-.673 1.5-1.5v-21c0-.017-.008-.031-.009-.047q-.004-.033-.013-.065a.5.5 0 0 0-.09-.191c-.007-.009-.006-.02-.013-.029l-8-9-.01-.006a.5.5 0 0 0-.223-.134q-.027-.008-.056-.011C15.557.012 15.53 0 15.5 0h-14C.673 0 0 .673 0 1.5v29c0 .827.673 1.5 1.5 1.5M16 1.815L22.387 9H16.5c-.22 0-.5-.42-.5-.75zM1 1.5a.5.5 0 0 1 .5-.5H15v7.25c0 .809.655 1.75 1.5 1.75H23v20.5a.5.5 0 0 1-.5.5h-21c-.28 0-.5-.22-.5-.5z"/><path d="M5.5 14h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1m0 4h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1m0-8h6a.5.5 0 0 0 0-1h-6a.5.5 0 0 0 0 1m0 12h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1m0 4h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1"/></g></symbol>
      <symbol id="i-dapp-assets-locker" viewBox="0 0 24 24" data-iconify="material-symbols-light:lock-outline" data-menu-icon-mode="normalized-fill" data-menu-icon-base-weight="1" data-menu-icon-weight="1.5"><path fill="currentColor" stroke="currentColor" stroke-width="0.5" stroke-linejoin="round" d="M6.616 21q-.672 0-1.144-.472T5 19.385v-8.77q0-.67.472-1.143Q5.944 9 6.616 9H8V7q0-1.671 1.165-2.835Q10.329 3 12 3t2.836 1.165T16 7v2h1.385q.67 0 1.143.472q.472.472.472 1.144v8.769q0 .67-.472 1.143q-.472.472-1.143.472zm0-1h10.769q.269 0 .442-.173t.173-.442v-8.77q0-.269-.173-.442T17.385 10H6.615q-.269 0-.442.173T6 10.616v8.769q0 .269.173.442t.443.173m6.45-3.934q.434-.433.434-1.066t-.434-1.066T12 13.5t-1.066.434Q10.5 14.367 10.5 15t.434 1.066q.433.434 1.066.434t1.066-.434M9 9h6V7q0-1.25-.875-2.125T12 4t-2.125.875T9 7zM6 20V10z"/></symbol>
      <symbol id="i-dapp-xchain-integration" viewBox="0 0 24 24" data-iconify="mdi-light:link-variant" data-menu-icon-mode="normalized-fill" data-menu-icon-base-weight="1" data-menu-icon-weight="1.5"><path fill="currentColor" stroke="currentColor" stroke-width="0.5" stroke-linejoin="round" d="M10.73 14.97c.27.11.36.41.24.66s-.41.37-.66.24h-.01c-.46-.21-.89-.51-1.27-.9a4.49 4.49 0 0 1 0-6.36l3.53-3.53a4.49 4.49 0 0 1 6.36 0a4.49 4.49 0 0 1 0 6.36l-1.63 1.63l-.15-1.26l1.08-1.08a3.513 3.513 0 0 0 0-4.95a3.513 3.513 0 0 0-4.95 0L9.73 9.32a3.513 3.513 0 0 0 0 4.95c.3.3.64.53 1 .7m-6.65 4.95a4.49 4.49 0 0 1 0-6.36l1.63-1.63l.15 1.26l-1.08 1.08a3.513 3.513 0 0 0 0 4.95a3.513 3.513 0 0 0 4.95 0l3.54-3.54a3.513 3.513 0 0 0 0-4.95c-.3-.3-.64-.53-1-.7v.01a.49.49 0 0 1-.24-.67c.12-.25.41-.37.66-.24h.01c.46.21.89.51 1.27.9a4.49 4.49 0 0 1 0 6.36l-3.53 3.53a4.49 4.49 0 0 1-6.36 0"/></symbol>
      <symbol id="i-aggregate" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="4.5" cy="5" r="1.5"/><circle cx="4.5" cy="12" r="1.5"/><circle cx="4.5" cy="19" r="1.5"/><path d="M6 5h2.5A3.5 3.5 0 0 1 12 8.5V12M6 19h2.5a3.5 3.5 0 0 0 3.5-3.5V12M6 12h13m-3-3 3 3-3 3"/></g></symbol>
      <symbol id="i-settings" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1a1.7 1.7 0 0 0 1.9.3A1.7 1.7 0 0 0 10 3V2.8h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z"/></symbol>
      <symbol id="i-advanced" viewBox="0 0 24 24"><path d="M4 6h10M18 6h2M4 18h2M10 18h10M4 12h4M12 12h8"/><circle cx="16" cy="6" r="2"/><circle cx="8" cy="18" r="2"/><circle cx="10" cy="12" r="2"/></symbol>
      <symbol id="i-send" viewBox="0 0 24 24"><path d="m5 19 14-14M9 5h10v10"/></symbol>
      <symbol id="i-receive" viewBox="0 0 24 24"><path d="m19 5-14 14M5 9v10h10"/></symbol>
      <symbol id="i-inbox" viewBox="0 0 24 24"><path class="icon-fill" d="M19 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2Zm0 12h-4a3 3 0 0 1-6 0H5V5h14v10Z"/></symbol>
      <symbol id="i-sent" viewBox="0 0 24 24"><path class="icon-fill" d="M3 20.4v-6.5l7-1.9-7-1.9V3.6L21 12 3 20.4Z"/></symbol>
      <symbol id="i-coin" viewBox="0 0 24 24"><circle cx="12" cy="12" r="8.5"/><path d="M14.5 8.5c-.6-.5-1.5-.8-2.5-.8-1.4 0-2.6.8-2.6 2s1 1.7 2.7 2.1c1.7.4 2.6.9 2.6 2.1 0 1.2-1.1 2-2.7 2-1.2 0-2.2-.4-2.8-1.1M12 6.2v11.6"/></symbol>
      <symbol id="i-token" viewBox="0 0 24 24"><path d="m12 3.5 7.5 4.3v8.4L12 20.5l-7.5-4.3V7.8Z"/><path d="M8.8 9.3h6.4M8.8 12h6.4M8.8 14.7h6.4"/></symbol>
      <symbol id="i-nft" viewBox="0 0 24 24"><path d="m12 3 8 9-8 9-8-9Z"/><path d="m8.5 12 3.5 3.5 3.5-3.5L12 8.5Z"/></symbol>
      <symbol id="i-voucher" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M3 5h18v5a2.5 2.5 0 0 0 0 5v4H3v-4a2.5 2.5 0 0 0 0-5Z"/><path d="M13 7.5v2M13 12v2M13 16.5v.25M7 12h3"/></g></symbol>
      <symbol id="i-right" viewBox="0 0 24 24"><circle cx="8" cy="12" r="3.5"/><path d="M11.5 12H21m-3 0v3m-3-3v3"/></symbol>
      <symbol id="i-claim" viewBox="0 0 24 24"><path d="M5 4h14v5a3 3 0 0 0 0 6v5H5v-5a3 3 0 0 0 0-6Z"/><path d="M12 7v10"/></symbol>
      <symbol id="i-import" viewBox="0 0 24 24" data-iconify="system-uicons:import" data-menu-icon-mode="outline" data-menu-icon-weight="1.8"><g transform="translate(-2 -2) scale(1.3333333333)" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.35"><path d="M9.5 3.5h-4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-10"/><path d="m13.5 10.5-3 3-3-3"/><path d="M17.5 3.5h-4a3 3 0 0 0-3 3v7"/></g></symbol>
      <symbol id="i-merge-split" viewBox="0 0 24 24" data-iconify="mdi-light:sitemap" data-menu-icon-mode="source-fill" data-menu-icon-weight="1.5"><path class="icon-fill" d="M9 3h5v5h-2v4h5a3 3 0 0 1 3 3v2h2v5h-5v-5h2v-2a2 2 0 0 0-2-2h-5v4h2v5H9v-5h2v-4H6a2 2 0 0 0-2 2v2h2v5H1v-5h2v-2a3 3 0 0 1 3-3h5V8H9zm4 4V4h-3v3zM5 21v-3H2v3zm8 0v-3h-3v3zm8 0v-3h-3v3z"/></symbol>
      <symbol id="i-merge" viewBox="0 0 24 24" data-iconify="material-symbols-light:merge" data-menu-icon-mode="normalized-fill" data-menu-icon-base-weight="1" data-menu-icon-weight="1.5"><path d="m6.4 20l-.688-.688l4.69-4.697q.633-.632.865-1.165t.233-1.429V5.883L9.38 7.996l-.688-.688L12 4l3.308 3.308l-.689.688L12.5 5.883v6.138q0 .896.252 1.448t.885 1.185l4.652 4.658L17.6 20L12 14.4z" fill="currentColor" stroke="currentColor" stroke-width=".5" stroke-linecap="round" stroke-linejoin="round" paint-order="stroke fill"/></symbol>
      <symbol id="i-split" viewBox="0 0 24 24" data-iconify="material-symbols-light:call-split" data-menu-icon-mode="normalized-fill" data-menu-icon-base-weight="1" data-menu-icon-weight="1.5"><path d="M11.5 19v-6.792L6 6.708V10H5V5h5v1H6.708l5.792 5.792V19zm2.658-8.439l-.72-.719L17.293 6H14V5h5v5h-1V6.708z" fill="currentColor" stroke="currentColor" stroke-width=".5" stroke-linecap="round" stroke-linejoin="round" paint-order="stroke fill"/></symbol>
      <symbol id="i-permission" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M12 2.75 20 6v5.5c0 4.7-3.25 8.2-8 9.75-4.75-1.55-8-5.05-8-9.75V6Z"/><circle cx="12" cy="10" r="2.25"/><path d="M12 12.25V17M12 15h2"/></g></symbol>
      <symbol id="i-eye" viewBox="0 0 24 24"><path d="M2.5 12s3.5-6 9.5-6 9.5 6 9.5 6-3.5 6-9.5 6-9.5-6-9.5-6Z"/><circle cx="12" cy="12" r="2.5"/></symbol>
      <symbol id="i-eye-off" viewBox="0 0 24 24"><path d="m3 3 18 18M10.6 6.1A11 11 0 0 1 12 6c6 0 9.5 6 9.5 6a16 16 0 0 1-2.2 2.8M6.2 6.2C3.8 8 2.5 12 2.5 12s3.5 6 9.5 6a9.8 9.8 0 0 0 3-.5"/><path d="M10 10a2.8 2.8 0 0 0 4 4"/></symbol>
      <symbol id="i-lock" viewBox="0 0 24 24"><rect x="4" y="10" width="16" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/></symbol>
      <symbol id="i-shield" viewBox="0 0 24 24"><path d="M12 3 4 6v5c0 5 3.4 8.5 8 10 4.6-1.5 8-5 8-10V6Z"/><path d="m8.5 12 2.2 2.2 4.8-5"/></symbol>
      <symbol id="i-bell" viewBox="0 0 24 24"><path d="M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9ZM10 21h4"/></symbol>
      <symbol id="i-user" viewBox="0 0 24 24"><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></symbol>
      <symbol id="i-message" viewBox="0 0 24 24"><path d="M4 5h16v11H9l-5 4Z"/><path d="M8 9h8M8 12h5"/></symbol>
      <symbol id="i-question" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><path d="M9.7 9a2.5 2.5 0 0 1 4.8 1c0 2-2.5 2.2-2.5 4M12 17.5v.1"/></symbol>
      <symbol id="i-chevron" viewBox="0 0 24 24"><path d="m9 6 6 6-6 6"/></symbol>
      <symbol id="i-copy" viewBox="0 0 24 24"><rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/></symbol>
      <symbol id="i-check" viewBox="0 0 24 24"><path d="m5 12 4 4L19 6"/></symbol>
      <symbol id="i-close" viewBox="0 0 24 24"><path d="m6 6 12 12M18 6 6 18"/></symbol>
      <symbol id="i-moon" viewBox="0 0 24 24"><path d="M20 15.5A8.5 8.5 0 0 1 8.5 4 8.5 8.5 0 1 0 20 15.5Z"/></symbol>
      <symbol id="i-sun" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></symbol>
      <symbol id="i-more" viewBox="0 0 24 24"><circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/></symbol>
      <symbol id="i-alert" viewBox="0 0 24 24"><path d="M12 3 2.5 20h19Z"/><path d="M12 9v5M12 17.5v.1"/></symbol>
      <symbol id="i-search" viewBox="0 0 24 24"><circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/></symbol>
      <symbol id="i-backup" viewBox="0 0 24 24"><path d="M12 3v11m-4-4 4 4 4-4"/><path d="M5 15v5h14v-5"/></symbol>
      <symbol id="i-restore" viewBox="0 0 24 24"><path d="M12 17V6m-4 4 4-4 4 4"/><path d="M5 14v6h14v-6"/></symbol>
      <symbol id="i-network" viewBox="0 0 24 24"><path d="M4 9a12 12 0 0 1 16 0M7 12.5a7 7 0 0 1 10 0M10 16a3 3 0 0 1 4 0"/><circle cx="12" cy="19" r="1"/></symbol>
      <symbol id="i-overview" viewBox="0 0 24 24"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></symbol>
      <symbol id="i-reticulum-node" viewBox="0 0 24 24"><circle cx="12" cy="5" r="2.5"/><circle cx="5" cy="19" r="2.5"/><circle cx="19" cy="19" r="2.5"/><path d="M12 7.5v4.5M10.5 13.5 6.5 17M13.5 13.5l4 3.5"/></symbol>
      <symbol id="i-reticulum-interface" viewBox="0 0 24 24"><rect x="3" y="6" width="18" height="13" rx="2"/><path d="M7 10h10M7 14h5M7 3v3M17 3v3"/></symbol>
      <symbol id="i-entry" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" d="M17 12H1m10.5-5c0 .577.665 1.562 1.228 2.294a7.5 7.5 0 0 0 1.745 1.662C15.2 11.445 16.2 12 16.99 12c-.79 0-1.79.556-2.517 1.044a7.5 7.5 0 0 0-1.745 1.662c-.563.732-1.228 1.717-1.228 2.294m-3-10V2.5h.329A46 46 0 0 0 21.897.605L22.25.5h.25v23h-.25l-.353-.105A46 46 0 0 0 8.829 21.5H8.5V17"/></symbol>
      <symbol id="i-reticulum-paths" viewBox="0 0 24 24"><circle cx="5" cy="5" r="2"/><circle cx="19" cy="7" r="2"/><circle cx="7" cy="19" r="2"/><path d="M7 5h3a4 4 0 0 1 4 4v3a4 4 0 0 0 4 4h1M7 19h2a5 5 0 0 0 5-5V9a2 2 0 0 1 2-2h1"/></symbol>
      <symbol id="i-queue" viewBox="0 0 24 24"><path d="M4 6h16M4 12h11M4 18h11M18 14l3 3-3 3"/></symbol>
      <symbol id="i-probe" viewBox="0 0 24 24"><path d="M10 5a3 3 0 0 1 6 0v8.2a5 5 0 1 1-6 0Z"/><path d="M13 6v9M13 18h.01"/></symbol>
      <symbol id="i-reticulum-link" viewBox="0 0 24 24"><path d="M10 13a5 5 0 0 0 7.5.5l2-2a5 5 0 0 0-7-7l-1.2 1.2M14 11a5 5 0 0 0-7.5-.5l-2 2a5 5 0 0 0 7 7l1.2-1.2"/></symbol>
      <symbol id="i-plus" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></symbol>
      <symbol id="i-logout" viewBox="0 0 24 24"><path d="M10 5H5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h5"/><path d="m14 16 4-4-4-4M18 12H9"/></symbol>
      <symbol id="i-remove" viewBox="0 0 24 24"><path d="M4 7h16M10 11v6m4-6v6M9 7l1-3h4l1 3M6 7l1 13h10l1-13"/></symbol>
      <symbol id="i-storage" viewBox="0 0 24 24"><ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v7c0 1.7 3.6 3 8 3s8-1.3 8-3V5M4 12v7c0 1.7 3.6 3 8 3s8-1.3 8-3v-7"/></symbol>
      <symbol id="i-bar-chart" viewBox="0 0 24 24"><path d="M4 20V11h4v9M10 20V5h4v15M16 20v-6h4v6M3 20h18"/></symbol>
      <symbol id="i-line-chart" viewBox="0 0 24 24"><path d="M3 20h18M4 16l5-5 4 3 7-8"/><circle cx="4" cy="16" r="1"/><circle cx="9" cy="11" r="1"/><circle cx="13" cy="14" r="1"/><circle cx="20" cy="6" r="1"/></symbol>
      <symbol id="i-info" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7.5v.1"/></symbol>
<symbol id="i-page-outline" viewBox="0 0 24 24"><path d="M6 4.2h12v15.6H6zM9.6 8.4h4.8M9.6 12h4.8M9.6 15.6h3"/></symbol>
  `;
  const ICON_NAMES = Object.freeze(
    [...ICON_SPRITE_MARKUP.matchAll(/<symbol\s+id="i-([^"]+)"/g)].map((match) => match[1])
  );

  function mountIconSprite(doc = root.document) {
    if (!doc?.body || doc.querySelector("[data-z00z-icon-sprite]")) return;
    const template = doc.createElement("template");
    template.innerHTML = `<svg class="svg-sprite" data-z00z-icon-sprite aria-hidden="true">${ICON_SPRITE_MARKUP}</svg>`;
    doc.body.prepend(template.content.firstElementChild);
  }

  Object.assign(root.Z00ZDemo ||= {}, {
    ICON_NAMES,
    ICON_SPRITE_MARKUP,
    mountIconSprite
  });

  if (root.document) mountIconSprite(root.document);
})(typeof window === "undefined" ? globalThis : window);
