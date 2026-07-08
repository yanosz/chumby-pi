# 20 — Info & Licenses + live WLAN signal (E6/E7)

Working record for the milestone scoped 2026-07-08 (plan: "Milestone: Info &
Licenses + live WLAN signal (E6/E7)"). Three increments I1/I2/I3, each with a
CHECKPOINT. Screens E6 (Info/About) and E7 (Licenses); plus the dashboard WLAN
signal bar. This doc accumulates findings + verification per increment.

## Screen mechanics (from the ffdec export, controlpanel-2.8.87b3)

- **Nav path:** dashboard → (bend → main bar B2) → SETTINGS → CHUMBY INFO icon →
  `InfoPanel` (E6) → SOFTWARE LICENSE → `LicensePanel` (E7). The Info icon is a
  Settings-menu child; Licenses is reachable ONLY through the Info screen.
- **`InfoPanel`** (`frame_2` ~27130): `loadInfo()` refreshes every 60 frames and
  prints id/HW/SW/FW/HW#, a `registered to / chumby name / channel name` block
  gated on `g_authorized && hasNetwork`, then MAC/type/ssid/ip/netmask/gateway/
  dns, and for `networkType=="wlan"` a `link quality/signal/noise` line from the
  `signal_strength` backtick. Buttons: `licenseButton` (SOFTWARE LICENSE),
  `piButton` (the "π" top-right — the **geek trigger**, `geekMode()`),
  `introButton` (`player.playIntro()`, visible when `hasNetwork`).
- **`LicensePanel`** (`frame_2` ~27407): `gplButton`/`lgplButton` call
  `loadFile("gpl"|"lgpl")`, which does `XML.load("file:////LICENSES/"+l+".txt")`
  (the `_root.test` branch — not our mode — would use a relative path). Opens on
  `loadFile("gpl")`. It never loads `README`. The `onData` handler drops the raw
  text into a scrolling TextField.

## file:// resolution (the I1 plumbing problem)

Widget SWFs/thumbnails are referenced as `file:///{FIXTURES}/widgets/…`; the
`{FIXTURES}` token expands (in `fixture.rs::expand_tokens`) to the absolute
fixtures dir, so those are **real disk paths** loaded by the stock navigator.
The licenses viewer instead hardcodes `file:////LICENSES/gpl.txt` (a chumby
rootfs path we cannot change — rule 3), which the stock navigator would resolve
to real-disk `/LICENSES/…` and never find. Fix: `ChumbyNavigator::intercept`
serves `file://` loads from the virtual rootfs first, falling through to the
real navigator on a miss (so `{FIXTURES}` real paths are unaffected). This makes
XML.load(file://) consistent with the fs natives and also covers any future
panel-hardcoded file:// read (e.g. the E9 file browser).

## I1 — Licenses (E7) — DONE (desktop) 2026-07-08

Changes:
- `fixtures/rootfs/LICENSES/{gpl.txt,lgpl.txt,README}` — copied **verbatim** from
  `/home/jan/chumby_backup/LICENSES/` (sha256 match; license texts are freely
  distributable, so committed — not gitignored like the SWFs/tones). `README` is
  shipped for directory fidelity though the panel never displays it.
- `ruffle/core/src/chumby/navigator.rs` — `file://` → rootfs interception (above;
  patch-notes behavior note).
- `fixtures/ui-policy.toml` — dropped the `settings-info` rule so the Info icon
  (the gateway to E6/E7) is reachable again. NB the info screen's own geek
  (`piButton`) + intro (`introButton`) buttons get disabled in I2, not here; in
  the I1 state they are briefly live (same "reachable dead-end until disabled"
  tradeoff the widget milestone accepted).

Verification (desktop, freshly built `ruffle_desktop`, `-PlocalCache=1`, 640×480):
drove the control channel through SETTINGS → CHUMBY INFO → SOFTWARE LICENSE.
- Log: `file:// rootfs HIT file:////LICENSES/gpl.txt`, and after tapping LGPL,
  `…/LICENSES/lgpl.txt` — the viewer requested each and the rootfs served it.
- Screens: GPL renders ("GNU GENERAL PUBLIC LICENSE / Version 2, June 1991 …"),
  LGPL swaps in ("GNU LESSER GENERAL PUBLIC LICENSE / Version 2.1 …"). GPL/LGPL/
  DONE + scroll arrows present.
- No regression: the widget SWF still loaded via passthrough
  (`file:////home/jan/chumby-pi/fixtures/widgets/unsubscribedclock.swf`).

Observations parked for later increments (seen on the Info screen during the I1
walk-through):
- The **registration block shows** fixture values (`registered to: jan`,
  `chumby name: chumbypi`, `channel name: chumbypi-channel`) because the
  authorize-bypass fixture reports `g_authorized=1`. GUID/registration is out of
  scope (user), so this is fixture data — decide in I2 whether to leave it or
  blank it.
- `ssid:` is empty and no wifi link-quality line shows → the network fixture
  reports a non-wlan type; the dashboard shows full green signal bars (the
  static `signal_strength` fixture). Both go real in I3.

## I3 planning note — the deploy device is WIRED (3B+), + the "ethernet bar"

Corrected 2026-07-08 (Jan): the running device is a Pi **3B+ on WIRED
Ethernet** (not the interim 3A+/wifi-only from doc 07). So I3's real
`network-status` reports **`type=lan`** (Info screen E6 already renders
"type: Ethernet / ip / netmask / gateway / dns" from that).

Dashboard indicator on wired: the only network element is the
`WifiIndicator` (class `frame_2:8060`, placed in `DefineSprite_776`) — a
5-segment meter (`WifiSignalStrengthIndBar` bars, each `gotoAndStop
("on"|"off")`, `frame_2:28646`). It shows only when the `signal_strength`
XML has `connected="1"`, else hides; level = `(linkquality-50)*2` in 20%
steps. There is **no wired/ethernet icon anywhere in the SWF**.

Decision (Jan, 2026-07-08): repurpose the meter as an **ethernet
indicator** — `signal_strength` (PiHost) returns `connected=1
linkquality=100` on a healthy wired link (5 bars shown), and **recolour**
the bars to a distinct non-wifi colour. Recolour is feasible without
touching the SWF: the panel already tints clips via `new Color(mc).setRGB
(...)` (e.g. buttons `frame_2:6833`), proven in our player; we drive the
same from the host as a new **`tint`** action in the ui-policy mechanism,
applied to the parent `WifiIndicator` (a child bar's `gotoAndStop` won't
reset a parent colour transform), re-applied on frame entry. Need the
`WifiIndicator` selector (find via `chumby_pick=debug`). Honesty: distinct
colour + the Info screen's "type: Ethernet" line.

On-device (2026-07-08, Jan's request — earlier than the plan's I3 pass):
fork `86ae7f86a` hot-replaced to the Pi (`192.168.42.30`) — fresh aarch64
dist binary + fixtures; service active, LICENSES seeded, Info icon re-enabled,
no panics. Full record: doc 09 "Hot-replace deploy, 2026-07-08 (the I1
Licenses build)". Visual confirmation of the viewer on the TFT left to Jan
(the B2 main bar auto-hides, so scripted navigation over SSH is fiddly).

## I3 — Real network diagnostics + ethernet bar — DONE (desktop) 2026-07-08

Fork `86ae7f86a` → `dec173b47`. Three requirements (Jan 2026-07-08), all
desktop-verified with `--chumby-real-net` on the dev box:
1. **Blue meter for wired.** `signal_strength` → `connected=1 linkquality=100`
   (5 bars) + a new ui-policy `tint` rule recolours the `WifiIndicator` blue.
2. **Real IP of the connected interface.** `network_status.sh` → real
   ip/netmask/gateway/dns; `macgen.sh` → real MAC.
3. **Honest lan page** (Jan's pick over the ssid/speed trick): `type="lan"`
   → the Info screen shows "type: Ethernet" + real ip/gw/dns, no ssid row and
   no wifi link-quality line.

Implementation:
- `chumby/real_net.rs` — `RealNetHost` wraps `FixtureHost` and overrides the
  `exec` touchpoints `network_status.sh` / `signal_strength` / `macgen.sh`
  from live kernel state: default-route interface + gateway from
  `/proc/net/route`, the interface's IPv4 + netmask from **`getifaddrs`**, DNS
  from `/etc/resolv.conf`, MAC from `/sys/class/net/<if>/address`. No shell
  (design principle). Assumes one connected interface (Jan).
  **REVISED 2026-07-08 (fork `dec173b47`→`b9199a1f6`, user):** dropped the
  `--chumby-real-net` flag — `RealNetHost` is now **always active**, and a
  read that finds no interface returns `None` → falls back to the inner
  fixture (so an offline desktop/CI run is unchanged). Also replaced the two
  brittle bits (a UDP `local_addr` probe for the IP, and a `/proc/net/route`
  hex-parse for the netmask) with `getifaddrs` — the canonical source, robust
  to the edge cases the probe/parse weren't. `getifaddrs` uses `libc`, added
  target-gated (`cfg(unix)`) to `core/Cargo.toml`; a `cfg(not(unix))` stub
  keeps wasm building.
- `chumby/ui_policy.rs` — new `Tint(0xRRGGBB)` action (AVM1 `Color.setRGB`
  flat-fill via `set_color_transform`), parsed from `color = "#RRGGBB"`.
- `fixtures/ui-policy.toml` — rule `wired-eth-bar`, `color="#1E90FF"`,
  selector `controlPanel/name:mainButtons/depth:136` (the WifiIndicator,
  chid 776, is placed UNNAMED at depth 136 in `mainButtons`/DS784 — depth
  selector required).
- `fixtures/exec/network_status.xml` — desktop fixture aligned to `type="lan"`
  so the desktop info page mirrors the honest wired page (real-net off).

Desktop evidence: main-bar meter renders **blue** (was green); `wired-eth-bar`
log `target acquired — applying Tint(0x1E90FF)`; Info page shows the dev box's
real `ip 192.168.42.70`, gw/dns `192.168.42.1`, netmask `255.255.255.0`, MAC
`bc:24:11:75:e4:56`, `type: Ethernet`, no wifi line.

**Remaining (deferred — session paused 2026-07-08):** the on-device pass —
aarch64 dist rebuild at `dec173b47`, add `--chumby-real-net` to the Pi
launcher (`chumby-player-run`, a Pi op to document), deploy, confirm the Pi's
real `eth0` IP + blue bar on the TFT. Then feature-decisions network-class row
(signal_strength/network-status → real on Pi) + CI movie-start green close I3.
Also still open: I2's `piButton`/`introButton` disable + the registration-block
leave/blank decision (the fixture "registered to: jan" block still shows).
