# 18 — Clock screens catalog & UI-policy mechanism (milestone step 1)

Date: 2026-07-07. Input for CHECKPOINT UI1 (plan: "UI policy" milestone).
Decision being implemented: the Pi OS owns timezone + NTP (real chumby
verified to run in the user's local TZ), so the panel shows the timezone
read-only, TZ picker + NTP selection are disabled, only the 12/24h
checkbox stays functional.

Sources: ffdec swf2xml dump of `swf-assets/controlpanel.swf` (2.8.87b3,
instance names/depths), the script export (appendix, class code), and a
live desktop fixture run 2026-07-07 (screenshots
`images/18-e5-clock-panel.png`, `images/18-e12-timezone-map.png`).

## 1. Reachability: B3 is dead UI in 2.8.87b3

The screen catalog (05) lists two clock screens: B3 (`controlPanel`
frame `clock`, DS1145 `ClockPanel`) and E5 (Settings → TIME/DATE,
DS1736 `ClockOnlyPanel`). Finding: **B3 is unreachable in this
variant.**

- The only navigation to the B3 frame is `DS1766 frame_2` function
  `clock()` — whose only caller is `clockButton.onButtonRelease` wired
  in the B2 constructor (frame_2/DoAction.as:7986).
- But the B2 sprite (DS784 `mainButtons`) **places no `clockButton`**
  (full child list checked in the XML dump) — the handler is assigned
  to a nonexistent child, i.e. dead code from an older layout.
- Live run confirms: the B2 bar shows MUTE/CHANNEL/NIGHT/MUSIC/
  SETTINGS/ALARMS + STAY/SEND/RATE/DELETE, no Clock.

Consequence: **only the E5 chain and the E12 map need policy
entries.** B3's internals (DS1145 → DS658 `ClockPanelMain`) are
cataloged in §2c for completeness; note its `refreshDate` hardcodes
12-hour format (`%I:%M:%S %P`) — no `mode24Button` exists there. The
12/24h toggle exists **only on E5**.

## 2. Display-tree catalog

### 2a. E5 — Settings → TIME/DATE (`ClockOnlyPanel`)

Chain from the root (instance names from PlaceObject tags; "unnamed" =
no name attribute → AVM1 auto-names `instanceN`, load-order dependent):

| level | sprite | placed in | frame (label) | depth | instance name | class |
|---|---|---|---|---|---|---|
| 1 | DS1766 | main timeline | 6 (`main`) | 4 | `controlPanel` | screen switcher |
| 2 | DS1748 | DS1766 | 9 (`settings`) | 15 | **unnamed** | settings grid (E0) |
| 3 | DS1736 | DS1748 | 11 (`clock`) | 1 | **unnamed** | `ClockOnlyPanel` |
| 4 | DS1735 | DS1736 | 1 (`clockPanel`) | 1 | `timePanel` | `ClockOnlyPanelMain` |

DS1736 frames (screen-internal navigation): 1 `clockPanel`,
2 `setTimePanel` (DS666 `setTimePanel` + `doneButton`), 3 `setDatePanel`
(DS685 `setDatePanel`), 4 `setTimeZonePanel` (DS627, **unnamed**).

Controls in DS1735 (`…timePanel`), depth order:

| depth | char | instance | kind | behavior / touchpoints | policy proposal |
|---|---|---|---|---|---|
| 1 | 395 | `doneButton` | SmartTextButton | persists `putfile /psp/clock_format`, exits | keep |
| 13 | 1728 | – | art | screen bg | – |
| 14 | 1729 | `mode24Label` | text | "24 hour mode" | keep |
| 15 | 161 | `mode24Button` | CPCheckBox | toggles `g_clockFormat` 12↔24 (persist on done) | **keep (the one live setting)** |
| 21 | 1730 | `timezoneStr` | text | shows `/psp/timezone_city` city | keep (read-only display) |
| 22 | 1731 | `dateStr` | text | date display | keep |
| 23 | 1732 | `timeStr` | text | time display, honors 12/24 | keep |
| 24 | 645 | `setTimezoneButton` | sprite button (inner `b`) | → frame `setTimeZonePanel` (E12) | **DISABLE** |
| 29 | 647 | – | grey art | grey SET TIME icon (shows when button hidden) | – |
| 32 | 649 | `setTimeButton` | sprite button | → `setTimePanel`; `_setSystemTime` (5,176) | self-hidden (see below) |
| 37 | 651 | – | grey art | grey SET DATE icon | – |
| 40 | 653 | `setDateButton` | sprite button | → `setDatePanel`; `_setSystemTime` | self-hidden |
| 45 | 1733 | – | grey art | grey checkbox (shows when `ntpButton` hidden) | – |
| 46 | 161 | `ntpButton` | CPCheckBox | on change: `exec://sync_time_state.sh 0|1`, then `fixButtons()` | **DISABLE** |
| 52 | 656 | `ntpLabel` | text | "Set time from the Internet" | keep (or dim with checkbox) |
| 55 | 226 | – | art | divider | – |
| 56 | 1734 | `screenLabel` | text | "Time/date" | keep |

Self-hiding already in the SWF (`fixButtons()`): with
`/psp/use_ntp = 1` (our seeded fixture), `setTimeButton` and
`setDateButton` are `_visible = false` and their grey twins (647/651)
show — confirmed in the live screenshot. **Keeping `use_ntp = 1`
seeded means the SWF hides manual time/date setting by itself**; the
policy only has to guarantee the NTP checkbox can't be toggled off.

### 2b. E12 — the timezone map (`ClockPanelSetTimezone`, DS627)

Placed unnamed at depth 1 of DS1736 frame 4 (and of DS1145 frame 4 for
the orphaned B3; also used by the skipped wizard A2). Children:

| depth | char | instance | kind | behavior |
|---|---|---|---|---|
| 5 | 395 | `okButton` | SmartTextButton | `save()`: `_setTimeZone(filepath)` (5,178) + `putFile /psp/timezone_city` |
| 17 | 395 | `cancelButton` | SmartTextButton | exit without saving |
| 30 | 617 | `map` | zoomable world map | city-dot picking → `showLocation()` |
| 36 | 619 | `location` | text | picked "City, Country" |
| 37 | 625 | `zoomOutButton` | sprite button | map zoom out |
| 42 | 626 | `screenLabel` | text | "Set time zone" |

With `setTimezoneButton` disabled this screen becomes unreachable from
E5; no policy entries needed here (belt-and-braces row optional).

### 2c. B3 (orphaned) — `ClockPanel` DS1145 / `ClockPanelMain` DS658

For the record only. DS1145 placed as `controlPanel.clockPanel`
(DS1766 frame 3 `clock`, depth 15). DS658 = `…clockPanel.timePanel`,
same control set as DS1735 **minus** `mode24Button`/`mode24Label`,
**plus** `setAlarmsButton`, `setAlarm1Button`, `alarm1Str` (its alarm
frames place the same DS1119 alarms panel that B5 uses via B2→ALARMS).
Its NTP checkbox art twin is char 654. No policy entries (unreachable).

## 3. Component input mechanics (determines which actions work)

- **CPCheckBox (char 161** — `ntpButton`, `mode24Button`**)**: input =
  inner `box.onRelease`; `mark` = checkmark. Has a **built-in disabled
  state**: `enable(false)` → `gotoAndStop(2)`, and frame 2 **removes
  `box` and `mark` entirely** (grey art char 160) — input dead by
  construction. NOTE: AVM1 `enabled = false` on the *outer* clip does
  NOT block the inner `box` (enabled does not cascade); disable must
  target `box` or use the frame-2 state.
- **SmartTextButton (char 395** — done/ok/cancel**)**: labeled frames
  `enabled` (with `hotspot_mc`) / `disabled` (hit removed, grey art);
  `SmartTextButton.prototype.enable(e)` exists (frame_2 line 6838).
- **Simple sprite buttons (645/649/653…)**: inner `b.onRelease →
  _parent.onButtonRelease()`; **no disabled frame**. Disable = set
  `b.enabled = false` (+ dim `_alpha` on the outer clip for the look),
  or hide (`_visible = false`) — hiding reveals a grey twin only where
  the layout provides one (time/date yes; **no grey twin found for the
  TZ globe**).

## 4. Selector stability (user requirement)

Named instances (`controlPanel`, `timePanel`, `ntpButton`, …) are
stable — they come from PlaceObject tags in the SWF. The hazards are
the **unnamed links** in the chain (DS1748, DS1736, DS627): AVM1
auto-names them `instanceN` with a global counter that depends on how
many unnamed instances were created earlier in the session — i.e., on
the user's navigation history. **Dotted absolute paths through those
links are NOT stable.**

Selector proposal — per element, a list of alternatives tried in
order, first match wins, unresolved ⇒ logged WARNING:

- `name:` segment — child by instance name (stable when present);
- `depth:` segment — child by depth in the parent (stable: depths are
  authored in the SWF, verified above);
- segments compose a path from a stable anchor (`_root.controlPanel`).

Example (`ntpButton`, primary + fallback):
`controlPanel / depth:15 / depth:1 / name:timePanel / name:ntpButton`
and `controlPanel / depth:15 / depth:1 / depth:1 / depth:46`.

## 5. Proposed policy mechanism & config (for CHECKPOINT UI1)

Mechanism (per the confirmed direction, plan §"UI policy"): the
bootstrap controller applies the policy **on every frame-label change
of watched containers** (cheap; the same hook the wizard skip uses)
and re-applies idempotently, so SWF-side re-inits (`fixButtons()`,
screen re-entry) can't resurrect a control.

Config: `fixtures/ui-policy.toml` (versioned, chumby-pi repo):

```toml
# Disable NTP toggle: the Pi OS owns time sync (systemd-timesyncd).
[[rule]]
id        = "clock-ntp-toggle"
action    = "disable"           # hide | disable | readonly
selectors = [
  "controlPanel/depth:15/depth:1/name:timePanel/name:ntpButton",
  "controlPanel/depth:15/depth:1/depth:1/depth:46",
]

# Disable the timezone picker: TZ comes from the Pi OS.
[[rule]]
id        = "clock-set-timezone"
action    = "disable"
selectors = [
  "controlPanel/depth:15/depth:1/name:timePanel/name:setTimezoneButton",
  "controlPanel/depth:15/depth:1/depth:1/depth:24",
]
```

Action semantics (type-generic, per §3, implemented once in the
chumby module):

- `hide` — `_visible = false` (grey twin shows where the layout has
  one);
- `disable` — kill input **including inner hit clips** (`enabled =
  false` on the target and its children) + dim (`_alpha ≈ 45`) so the
  state reads as "shown, not changeable". For components with an
  authored disabled frame (CPCheckBox/SmartTextButton) the
  implementation MAY use it instead of dimming — decide at
  implementation with screenshots;
- `readonly` — input TextField → dynamic/unselectable (no target yet
  on these screens; kept for the general mechanism).

Rationale for `disable` over `hide` on both rules: NTP *is* active on
the Pi (systemd) — a checked-but-grey checkbox is truthful, while
hiding `ntpButton` would show the grey twin *unchecked* (the mark
lives inside the hidden clip), reading as "NTP off". For the TZ globe
there is no grey twin, so hiding would leave an empty gap; dimming
keeps the label + "Oslo" city string as the read-only display the
decision asks for.

## 6. Live verification (2026-07-07, desktop fixture run)

Route bend → SETTINGS (448,458) → TIME/DATE (125,316); fork
`6c99ebd1f`. Log confirms the contract rows:
`_getFile("/psp/timezone_city") -> "Oslo\tNorway"`,
`_getFile("/psp/use_ntp") -> "1"`, `use_ntp = 1` trace, then
`fixButtons` hides SET TIME / SET DATE (grey icons visible in
`images/18-e5-clock-panel.png`). Timezone map opened (SET TIME ZONE ≈
(518,256)) → `images/18-e12-timezone-map.png`, left via CANCEL — no
`_setTimeZone` call fired, fixtures tree verified clean afterwards.
Window-automation note: clicks silently miss unless the Ruffle window
is raised first (`xdotool windowraise` before `mousemove --window`) —
the pointer must land on Ruffle, not an overlapping window.

## 7. Side findings (recorded, no action)

- **B3 orphaned** (§1) — 05-screens.md's B2 button list includes
  "Clock" but 2.8.87b3's B2 sprite has no such child.
- **`sync_time_state.sh` has no exec fixture** (not in
  `fixtures/exec/manifest.txt`); today toggling NTP logs a MISSING
  exec and returns empty. Moot once `ntpButton` is disabled; add a
  fixture only if UI1 decides to keep the toggle live.
- **clock_format live-update gap** (Jan, on-device 2026-07-07, plan
  note): the 12/24h toggle persists but the running clock widget only
  picks it up at start — real hardware pushes
  `_setSlaveVar("_chumby_clock_format", …)` every heartbeat to the
  slave clock; our in-movie widget path has no slave-var →
  running-widget bridge. Decision: no fix, recorded.
- The `doneButton` on E5 persists `/psp/clock_format` on exit — the
  12/24h setting is only written when leaving the screen.

## 8. CHECKPOINT UI1 — decided (Jan, 2026-07-07)

1. **`disable` = dim + dead input** for both `ntpButton` and
   `setTimezoneButton` (truthful "on, not changeable" reading; no
   grey-twin/gap problems).
2. **`fixtures/ui-policy.toml`** as proposed (rules are data, ship
   with fixtures, no rebuild to edit).
3. **`ntpLabel` dims together with its checkbox** (reads as one
   disabled unit) — implemented as a third rule targeting the label.
4. **No belt-and-braces rules** — the policy stays at the minimal
   rule set; E12 unreachable once the globe is disabled, B3 orphaned
   (§1 records why).

## 9. Implementation & verification (same day)

Implemented as designed (fork `core/src/chumby/ui_policy.rs`, ~330
lines; format + action semantics in its module header):

- Loaded once at startup next to `set_host` (`desktop/main.rs` hook
  line extended); parsed with the workspace-pinned `toml` crate (one
  dependency line added to `core/Cargo.toml` — recorded in
  patch-notes.md).
- Applied from `avm::method` on every native call — the panel polls
  `_bent` per frame, giving frame-cadence re-application for free (no
  new upstream hook; same pattern as the `WidgetPlayer.onPress`
  surgery). Idempotent; screens re-entering get re-disabled.
- Property sets go through the AVM1 object interface (`_visible`,
  `enabled`, `_alpha`), `disable` also sets `enabled = false` on the
  target's direct children — necessary because AVM1 `enabled` does not
  cascade and the hit handlers sit on inner clips (`box`, `b`).
- Log damping is transition-based per rule (acquired / parent-only /
  gone); the "screen present but control missing — selectors stale?"
  warning fires only for the PRIMARY selector, because depth-based
  fallbacks legitimately walk into other frames of the same container
  (the settings grid reuses depths across frames) and would cry wolf.
- Unit test `ui_policy::tests::test_parse_rules_and_selectors`
  (parse + selector segmentation + malformed-rule skipping) passes.
- `fixtures/ui-policy.toml` ships the three UI1 rules; fixtures README
  row added.

Desktop verification (2026-07-07, pick-traced clicks — a first attempt
without `chumby_pick=debug` was inconclusive because a missed click is
indistinguishable from an inert control):

- Policy loads: `UI policy loaded: 3 rule(s)`; on entering the clock
  screen all three rules log `target acquired — applying Disable`;
  the pick trace shows the live auto-names (`instance31`/`instance33`)
  in the unnamed chain, confirming the depth-selector rationale.
- NTP checkbox + label and SET TIME ZONE globe render dimmed; 24h
  checkbox + DONE stay full-strength (screenshot
  `images/18-e5-policy-applied.png`).
- Inertness proven: pick trace shows clicks LANDING on
  `setTimezoneButton.b` and `ntpButton.box` with zero effect (no
  `setTimeZonePanel` navigation, no `sync_time_state` exec, checkbox
  unchanged).
- 12/24h round-trip intact: clicking `mode24Button` toggles the mark
  and the time display switches to 24h live (`20:17:39`); toggled
  back + DONE afterwards, fixtures tree restored byte-exact.
- 45 s movie-start check green with the policy active (exit 124,
  `_getPlatform`, no panic).

On-device check + CI on the amended fork commit: see plan status /
follow-up notes.
