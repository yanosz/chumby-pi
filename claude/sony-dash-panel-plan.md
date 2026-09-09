# The Sony Dash panel, and its theme mechanism

Draft 2026-09-09, scope settled with Jan the same day. Five steps, the same
shape as the classic work, re-aimed at the **Sony Dash** — with the **theme
mechanism as the point of the exercise**, ahead of panel selection.

Not blind: chumby's own Dash package has been fetched and measured (headers,
tag inventory, string scan). No decompile yet, no feature diff.
Sources: `forum.chumby.com/viewtopic.php?id=9752` (Duane, 2017-07-15, read via
the Wayback Machine), the Wayback CDX index of `files.chumby.com`, and the
package itself.

## Goal

Today the project runs one panel: **controlpanel.swf 2.8.87b3** for the
classic chumby, fetched by `pkg/chumby-player/chumby-download-firmware`
(`CP_QUERY`, l. 71-73), hardcoded at 320x240
(`claude-docs/requirements.md:516`).

Bring in the **Sony Dash panel and the theme system it carries** — so that a
theme is something the user can choose, and ultimately something the user can
*supply*, including from a USB stick. Panel selection (classic vs Dash) comes
along as the packaging that makes it reachable, but it is not the headline.

## Settled scope

| Question | Answer |
|---|---|
| A Dash to test on? | **No device.** Verify on the desktop against fixtures, then on the Pi. |
| Target display | **1024x600 DSI** (the box reports hostname `chumby-pi-3`, 192.168.42.24 on 2026-09-09; read it off the device, it moves). The 480x320 and 640x480 panels are out of scope here. |
| Sony's 1.7.1526 firmware | **Deferred.** Not needed to start; ask again if step 2 gets stuck. Buying a Dash stays on the table. |
| Artefacts | **`chumby-pi-internal`**, following the precedent below. |
| Themes | **The point.** Ahead of panel selection, including the USB `externalthemes.xml` route. |

Precedent for artefacts, from inspecting the archive: whole obtained trees
live in `resources/` (zurk's offline firmware, 803 MB, keeps its own README),
ffdec exports live in `docs/reference/appendix/<name>/` with the exact
commands recorded in that directory's `README.md`, and each panel SWF is
registered with size/md5/version in `docs/reference/01-invocations.md`.
Binaries are tracked there without ceremony. So: `resources/dash-chumby-hidc10-1.0.0/`
for the package verbatim, `appendix/controlpanel-dash-1.0.0/` and
`appendix/companions-dash-1.0.0/` for the exports.

## The artefact

The Sony Dash (HID-C10, plus the HID-B7 / HID-B70 siblings) is a Sony device
running chumby software. After Sony's forced 1.7.1604 update bricked many
HID-C10s in 2017, Duane published an unbrick kit — which works by dropping
`flashplayer.cfg` and `unbrick.swf` on a USB stick, confirming the Dash runs
the same chumby Flash player — and then a chumby package for the Dash:

    http://files.chumby.com/dash/chumby/chumby-hidc10-1.0.0.zip
    1 431 349 bytes, md5 4fd7b88a993125a0c3d82aca8fe5673c
    (fetched 2026-09-09 via web.archive.org/web/20190612153336id_/)

1.4 MB, self-contained, 2017-era rather than a 2011 firmware relic — and it
ships **the host-side scripts with it**, which is the host boundary in
readable form rather than inferred from the SWF: `start_control_panel`,
`install_chumby.sh`, `uninstall_chumby.sh`, `more_startup.sh`,
`start_opening_anim.sh`, `stop_flashplayer.sh`, `download_theme`,
`crontabs/root`, `default_hosts`, `flashplayer.cfg`, `chumby_version` (1.0.0),
`release_mode` (`production/`). Plus `controlpanel.swf`, `default_theme.swf`,
`default_theme_name.txt` ("Space Theme"), `default_opening.swf`, `movie.swf`,
`factorytest.swf`, two `.sig` files and `sample_photos/`.

### Measured, not assumed

| | classic 2.8.87b3 | Sony Dash 1.0.0 |
|---|---|---|
| stage | 320x240 | **860x480** |
| SWF version | 6 | **8** |
| frames | 10 | **1** |
| file size | 593 693 | 884 301 (2 763 121 uncompressed) |
| tags | 2 003 | 6 316 |
| `DoAction` / `DoInitAction` | 9 / 3 | 1 / **1 332** |
| AVM2 (`DoABC`) | 0 | **0** |

**It is AVM1** — no ABC tags, `ChumbyNative` appears 100 times, so it is the
same `ASnative(5,N)` surface the fork already dispatches, in an AS2 class
architecture. The natives approach transfers; the exact set is step 2's job.

`default_opening.swf` is 20 571 bytes — byte-size-identical to the chumby-8
firmware's `opening.swf`, and **not** the classic's (31 297, 132 frames, which
is what the Plymouth boot theme is built against).

`default_theme.swf` is a small sibling of the panel: 860x480, 47 634 bytes,
428 tags, 91 `DoInitAction`, no ABC. So a theme is a real SWF program, not
an asset bundle.

### The theme mechanism, as far as strings show

The panel carries a catalog, not just a setting: `ThemeCatalog`,
`ThemeCatalogService`, `ThemeCatalogItem`, `ThemeCatalogItemLayout`,
`ThemeAssetInfo`, `ITheme`, `IThemeCallbacks`, `ShowTheme`/`HideTheme`,
`StartupPanelUpdateTheme`, and a `themeDoesNotSupportWidgetBrowser` flag.

Paths it names: `/psp/theme.swf`, `/psp/theme.swf.sig`, `/psp/theme_name.txt`,
`/psp/download_theme`, `/usr/widgets/theme.swf`, `/tmp/theme.swf`,
`/tmp/theme.swf.sig`, `/tmp/alttheme.swf`, and **`/mnt/usb/externalthemes.xml`**.

`download_theme` (60 lines of Perl, same shape as the classic's `download_cp`)
takes `<url> <md5>`, fetches to `/tmp/theme.swf`, checks the md5, and reports
back as XML.

Two `.sig` files ride along with the package, next to `movie.swf` and
`factorytest.swf`: 1 698 bytes each, and their content is a PEM
`-----BEGIN PUBLIC KEY-----` block (DSA parameters), not a detached signature.
The panel's string table also names `/psp/theme.swf.sig` and
`/tmp/theme.swf.sig`. **That is the whole of what is established** — nothing
here says what reads these, whether anything verifies anything, or whether the
theme `.sig` is even the same kind of file. Step 2 finds out.

## What we do not know

- Which of the 32 package files the panel reads at runtime, and which
  `exec://` strings it issues. The scripts are right there — step 2.
- Whether the theme catalog is served from anywhere that still answers, and
  the `externalthemes.xml` schema.
- What the `.sig` files are for, whether anything reads them, and what
  that means for supplying a theme of our own.
- What 860x480 costs on the 1024x600 DSI panel. The display is already the
  fps ceiling (`claude/issues.md` #4, ~6-7 fps) and this is 5.4x the classic's
  pixel area.

## Step 1 result — done 2026-09-09

**Parked.** `chumby-pi-internal/resources/dash-chumby-hidc10-1.0.0/` holds the
zip verbatim (sha256 `236397c7e3cf…`) and `package/`, its unpacked contents,
with a README carrying provenance, the forum thread, per-SWF hashes and the
inventory of all 34 files. Read-only ground truth, the way `chumby_backup` is
for the classic.

**Exported.** ffdec v26.2.1 into
`docs/reference/appendix/controlpanel-dash-1.0.0/` (12 MB — 1 447 `.as`, of
which **777 are AS2 class bodies** under `__Packages/`, plus `frames/`,
`texts/`, `tag-dump.txt`) and `.../companions-dash-1.0.0/` (2.3 MB —
`default_theme`, `movie`, `factorytest`, `default_opening`). Exact commands
appended to the appendix README; images/shapes/sprites deliberately skipped
(1 877 sprites at 860x480).

**Registered.** `docs/reference/01-invocations.md` gained a dated addendum
listing the five Dash SWFs with hash, size, stage and SWF version — as
variants D-H, alongside the classic's A-C. I appended rather than edited: that
document is the 2026-06-12 classic survey and is left as it was.

### What the export shows about the theme mechanism

It is not a setting, it is a subsystem, and it is all here:

- `com/blueocty/themes/ITheme.as` and `IThemeCallbacks.as` — **the interface a
  theme SWF implements.** This is the contract a theme of our own would have
  to satisfy.
- `com/chumby/controlpanel/dash/themes/` — `ThemeLoader`, `ThemeCallbacks`,
  and `themenetwork/{ThemeCatalog, ThemeCatalogService, ThemeCatalogItem,
  ThemeCatalogItemLayout}`.
- `com/chumby/controlpanel/settings/themes/` — a whole UI: `ThemesPanel`,
  `ThemeSelectorDialog`, `ThemesPanelChooseTheme`, `ThemesPanelShowTheme`,
  `ThemesPanelLoadingTheme(s)`, `ThemesPanelUpdateTheme`, plus a
  `ThemeWizard` (`ChooseBackground`, `ChooseLayout`, `ChooseChannel`,
  `EnterName`, `TextEntry`) — which step 2 found to be a stub: every page
  only calls `next()`, nothing is written.
- `com/chumby/controlpanel/startup/StartupPanelUpdateTheme.as` and
  `com/chumby/controlpanel/dash/ThemePhotos.as`.

Reading these is step 2's first job; nothing above is a claim about behaviour,
only about what exists in the export.

### Other packages present

`com/chumby/` carries `accelerometer`, `alarm`, `browser`, `display`, `gui`,
`i18n`, `image`, `ipod`, `keyboard`, `music`, `network`, `photos`, `time`,
`usb`, `util`, `weather`, plus `ChumbyNative.as`, `ScreenDimensions.as`,
`BendSensor.as`, `BendTapper.as`, `DaughterCardID.as`. Third-party packages
too: `com/weather`, `com/accuweather`, `gov/noaa`, `com/adelavoice`,
`com/blueocty`, `mx/transitions`.

### Archive commit

Jan committed the archive side as `chumby-pi-internal` `ff8d798ea` the same
day. Step 2 verified the parked package (md5/sha256 as above, `package/`
equals the zip byte for byte) and the export counts, and corrected one
number: the zip holds **32 files** in two directories (`unzip -l` reports
34 entries because it lists the directories).

## Steps

Each step ends with the engineering record updated, and **stops** at its
checkpoint.

### Step 1 — obtain, park, disassemble

Half done: the package is fetched and measured. What remains:

- Park the package verbatim in `chumby-pi-internal/resources/dash-chumby-hidc10-1.0.0/`
  with a README carrying the URL, size, md5 and the forum thread.
- ffdec-export `controlpanel.swf`, `default_theme.swf`, `movie.swf`,
  `factorytest.swf`, `default_opening.swf` into
  `appendix/controlpanel-dash-1.0.0/` and `appendix/companions-dash-1.0.0/`
  (scripts/frames/texts + `-dumpSWF`), extend the appendix README with the
  exact commands, and register the SWFs in `01-invocations.md`.
- Skip images/shapes/sprites unless asked — 1 877 sprites at 860x480.

**CHECKPOINT 1** — the export exists; report its shape.

### Step 2 — the theme subsystem first, then the rest

Read the theme path end to end before anything else: `ThemeCatalog` and
`ThemeCatalogService`, what `StartupPanelUpdateTheme` does, the `/psp` and
`/mnt/usb` touchpoints, the `externalthemes.xml` schema, `download_theme`'s
callers, and **what checks the signature**. Produce the answer to "what does
it take to show a theme of our own".

Then the general diff against 2.8.87b3 — screens, `ASnative(5,N)` ids,
`exec://` strings, files read, endpoints — as a table with file:line on both
sides. Lands in `ruffle/claude/`.

Also in this step, because it can invalidate everything after it: **measure
860x480 on the DSI box.**

**CHECKPOINT 2** — the theme mechanism explained, the feature table, the fps
number.

#### Step 2 result — 2026-09-09

Record: `ruffle/claude/dash-panel-survey.md` (theme mechanism §1, feature
diff §2, desktop first contact §3, DSI box §4). The short version:

- **No signing needed.** `ThemeLoader` loads the first existing of
  `/tmp/theme.swf`, `/mnt/usb/theme.swf`, `/tmp/alttheme.swf`,
  `/psp/theme.swf`, `/usr/widgets/theme.swf` with no check at all
  (`ThemeLoader.as:12,61-89`). The `/usr/bin/verify … /etc/sony.pub` call
  exists at two sites, but the startup one is unreachable (no
  `gotoState(CHECK_THEME_STATE)` anywhere) and the other only fires from a
  scheduler event, and only for a theme whose md5 matches a catalog entry.
- A theme is an 860x480 AVM1 SWF whose frame 1 runs
  `Theme.main(<subclass>, this)`; the base class and the 35-handler
  contract are in the Space Theme export. The panel hands it a callbacks
  object (~60 methods) and the theme decides where the widget rectangle is.
- `externalthemes.xml` on the stick (present at panel start) replaces the
  `files.chumby.com` catalog; its `url` entries are `file:///…` paths that
  get `cp`'d to `/psp/theme.swf`. Only needed for picking from a list.
- The theme "wizard" writes nothing; the catalog host is not archived and
  did not answer one probe.
- Fork facts: 141 of the Dash's 147 native ids are already dispatched; the
  six missing are flip/LED/backlight and `_getWidgetNumber`. `file://`
  loads already resolve against the rootfs (`navigator.rs:47-67`). New
  semantics: widget-in-a-rectangle, `sys://` scheme, `chumbthumb`,
  `imgtool`, `list_mounts`, the panel self-updater to intercept.
- Desktop: the Dash panel runs 45 s without a panic under the July release
  binary and stops in the network wizard (`ap_scan`,
  `network_adapter_list.sh` missing); 2 267 AVM1 stack-underflow warnings
  in that time, undiagnosed.
- DSI box (survey §4): classic panel 11.8 fps / 82 % of a core; **Dash
  panel 12.0 fps / 53 %**; **Space Theme standalone 9.5 fps / 105 %**. The
  860x480 stage is not the problem — the theme's own per-frame work is, and
  that is what step 3 has to size first. Box restored to the classic panel
  afterwards (72 commits/6 s).

### Step 3 — two change lists

- **Fork (`ruffle/`)**: the natives and `exec://` touchpoints the theme path
  needs, the fixture files it reads (`/psp/theme*`, `chumby_version`,
  `release_mode`, …), whatever the `.sig` finding in step 2 turns out to require,
  host-boundary changes, and the
  renderer cost of the larger stage. Each item sized and placed on the patch
  surface (`ruffle/claude/patch-surface.md`).
- **This repo**: how a theme reaches `/psp`, how a user supplies one
  (USB stick vs downloader), `$STATE` layout for two panels, the
  launcher/`/etc/default` setting, packaging, docs.

Fork items into `ruffle/claude/issues.md`, appliance items into
`claude/issues.md`, cross-referenced from here.

**CHECKPOINT 3** — you pick scope and order.

#### Step 3 result — 2026-09-09

Two lists, one issue per item, each sized and placed on the patch surface.

**Fork** (`ruffle/claude/issues.md` 11–19): 11 the Space Theme costs a full
core on the DSI box (M, investigate first — it can invalidate the rest);
12 reach the home screen offline: `builtin=1` quits on the Dash, the live
route needs `securityQuestion`/`securityAnswer`, `/xml/authorize`, and the
new **XAPI** endpoint family that replaces `/xml/profiles` (L, the big one);
13 exec touchpoints, `tzdump` (the clock's DST source) and `list_mounts`
first, the theme copy strings in Rust like the widget cache (M); 14 widgets
composed inside a theme-chosen rectangle via the panel's own proxy branch
(M); 15 the `sys://` scheme (S); 16 `files.chumby.com` fixtures — the
self-updater to block, `themes.xml` as a local catalog (S–M); 17 six
unbound natives (S); 18 the stack-underflow warnings (S to diagnose);
19 classic-shaped subsystems to re-verify: alarm hooks, ui-policy, audio,
display stubs (M).

**Appliance** (`claude/issues.md` 14–17): 14 fetch and unpack the package
into `$STATE/dash/` (M); 15 `CHUMBY_PANEL` in `/etc/default`, a fixture tree
per panel (M); 16 theme supply — seeded `/psp/theme.swf`, USB `theme.swf`
through the existing mount link, then the picker via a generated local
catalog (S then M); 17 docs, packaging, CI, boot theme unchanged (S).

Proposed order for step 4: fork 11 → 12 → 13 (`tzdump`, `list_mounts`) →
14 → 16 → 17 → 15 → 18 → 19, with appliance 14–16 interleaved once the
panel reaches its home screen on the desktop. Everything sits in the
additions commit except a possible core fix from 18 and a render-scale knob
from 11.

### Step 4 — implement the fork side

Work the approved list one item at a time, in `ruffle/` on branch `dev`,
against the decompile as fact base. Each item: consumer list first, then the
change, then a desktop run against fixtures, then the record. The theme path
is the first item.

**CHECKPOINT 4** — per item, not just at the end.

### Step 5 — themes for the user, and panel selection

- The user-supplied theme route, whichever step 2 shows to be real: a seeded
  `/psp/theme.swf`, and the `/mnt/usb/externalthemes.xml` stick.
- `chumby-download-firmware`: which panel, and which theme, fetched and kept
  apart under `$STATE`.
- Launcher + `/etc/default/chumby-player`: an explicit, active setting naming
  panel and theme — no silent fallback default.
- Packaging, `docs/setup.md`, and the boot-theme step (the Dash's
  `default_opening.swf` is not the classic's `opening.swf`, and the Plymouth
  theme is written against the classic's 132 frames).

**CHECKPOINT 5** — a fresh-card install on the DSI box, each panel selected,
and a theme of ours displayed.

## Operational note

`files.`, `wiki.` and `forum.chumby.com` are one host (173.255.240.107);
`www.` is not. A 12-way parallel probe of that host got this IP blocked, and
it was still blocked hours later — the forum thread above had to be read from
the Wayback Machine. **Probe that host serially and sparingly**; the archive
answers most questions without touching it, and serves files via
`web.archive.org/web/<timestamp>id_/<url>`.
