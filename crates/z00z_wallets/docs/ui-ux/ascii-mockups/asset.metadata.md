### 1) Asset Metadata panel (read-only, egui-style “card”)

```text
┌────────────────────────────────── Asset Metadata ────────────────────────────┐
│ Asset:  Z00Z (Z00Z Asset)                                    [ ⟳ Re-fetch ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│  Symbol (ticker):        Z00Z                                      [ Copy ]  │
│  Name:                  Z00Z Asset                                 [ Copy ]  │
│  Decimals:              9                                                      │
│  Network:               Mainnet                                                 │
│  Asset ID:              z00z:asset:0x...A19F                        [ Copy ]  │
│  Contract / Address:    0x........................................  [ Copy ]  │
│                                                                              │
│  Icon:   [ preview ]   [ Change… ]                                           │
│                                                                              │
│  Source:  ● Verified registry  (last updated: 12:34:56)                      │
│  Tags:    [ Verified ]  [ Stable? ]  [ Bridged ]  [ Popular ]                │
│                                                                              │
│  [ Close ]                                     [ View details ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `asset.metadata` in context (inside Asset Details)

### 2) Metadata section (collapsing header)

```text
┌──────────────────────────── Asset Details ▸ Technical ────────────────────────┐
│  ▾ Metadata                                                                  │
│                                                                              │
│  Symbol:     Z00Z                        Decimals:  9                        │
│  Name:       Z00Z Asset                  Network:   Mainnet                  │
│                                                                              │
│  Asset ID:   z00z:asset:0x...A19F                        [ Copy ] [⟳]        │
│                                                                              │
│  Contract:   0x.............................................. [ Copy ]       │
│                                                                              │
│  Source:     ● Verified registry                                              │
│  Tags:       Verified, Primary                                                │
│                                                                              │
│  [ Edit display settings ▸ ]                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Custom/unverified asset metadata (editable)

### 3) Metadata editor (for custom assets)

```text
┌────────────────────────────── Asset Metadata ▸ Edit ─────────────────────────┐
│ Asset:  ABC (Custom)                                           Source: ○ User │
├──────────────────────────────────────────────────────────────────────────────┤
│  Symbol:        [ ABC            ]                                           │
│  Name:          [ ABC Token      ]                                           │
│  Decimals:      [ 9         ▾    ]                                           │
│  Network:       [ Mainnet    ▾   ]                                           │
│  Asset ID:      [ z00z:asset:0x... ]   [ Validate ]                          │
│  Contract:      [ 0x............. ]                                          │
│                                                                              │
│  Icon:  [ Choose… ]   Preview: [ ]                                           │
│                                                                              │
│  Warning: Unverified metadata may lead to wrong amounts shown.               │
│  [ ] I understand                                                            │
│                                                                              │
│  [ Cancel ]                               [ Save ]                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Fetch states

### 4) Loading metadata

```text
┌────────────────────────────────── Asset Metadata ────────────────────────────┐
│ Fetching metadata…   ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                       │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5) Error fetching metadata (node / registry fail)

```text
┌────────────────────────────────── Asset Metadata ────────────────────────────┐
│ ⊗ Failed to fetch metadata                                                   │
│ Reason: registry unreachable / invalid asset id / timeout                     │
│                                                                              │
│ [ Retry ]                          [ Network settings ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Quick copy popup (optional, egui-style small overlay)

### 6) “Copied” toast

```text
┌───────────────────────────────┐
│ ● Copied: Asset ID            │
└───────────────────────────────┘
```