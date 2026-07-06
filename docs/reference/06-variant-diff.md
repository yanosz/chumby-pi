# 06 — Variant diff (Step 1.6, descoped)

Date: 2026-06-12. **Scope note:** at CHECKPOINT 1 the user decided to focus
exclusively on the 2.8.87b3 panel and avoid decompiling other variants. The
plan's script-level SWF diff is therefore *not done* (deliberately — written
down here per the artifact rule). What remains fully answerable without
decompiling: variant identification, the outside-the-SWF diff, and zurk's
impersonation architecture — which is the actual prior art we need.

## 1. Variant identification (final)

| Variant | File | md5 | Version | How identified |
|---------|------|-----|---------|----------------|
| Shipped builtin | `chumby_backup/usr/widgets/controlpanel.swf` | `1670093769…` | unknown (firmware 1.7.2 era, ≤2.8.x, "old firmware" per its own wizard trace) | not decompiled (descoped) |
| Downloaded | `chumby_backup/tmp/controlpanel.swf` | `21d54cd7f1…` | **2.8.87b3** | `/tmp/controlpanelversion` + banner trace + byte-identical to live chumby.com download (01 §1) |
| Zurk offline | `zurks…/www/controlpanel.swf` (+3 copies) | `81b58e0930…` | **2.8.84** | zurk's own `cgi-bin/controlpanel.sh` serves exactly this md5 with `build="2.8.84"` |

So zurk ships the (claimed-official, plausibly unmodified) **2.8.84** panel —
one minor release older than our 2.8.87b3. Community datapoints (forum
id=8326) match: 2.8.84 was the last broadly working classic panel before
2.8.87. Whether zurk's binary is bit-identical to Chumby's official 2.8.84
release is unverified (no independent copy to hash against) — but his whole
approach (below) is environment impersonation, not SWF patching, so stock is
likely. **Open question, acceptable risk.**

## 2. Outside-the-SWF diff: backup (1.7.2 ironforge) vs zurk's scripts

Generation caveat from 01/02 applies: zurk's `scripts/` come from a later
multi-platform firmware; not a clean A/B of same-generation scripts.
Differences that matter to us:

| Area | Backup (ground truth) | Zurk / later firmware |
|------|----------------------|----------------------|
| `download_cp` | hardcoded `www.chumby.com` | **`urlbase` override via `/psp/urlbase_dlcp`**, version spoof via `/psp/cp_software_ver`/`cp_firmware_ver`, `VIDEO_RES`-aware splash, brand support |
| `start_control_panel` | as documented in 01 §4.1 | + demo-mode movies, + `--MaxLocalMemBlocksSlave=384` |
| `start_opening_anim` | `-L 0 -Q` | `-L 0 -Q -3` (no btplayd) |
| `signal_soft_event.sh` | `chumbyflashplayer.x -F1` | `killall -HUP chumbyflashplayer.x` |
| extra scripts | — | `signal_usb_event.sh`, `usb_keyboard_egg.sh`, update machinery (OTA), toolchain shims, `ytextract.pl` |
| web CGIs | memstats only | full remote control (`zmote_*`, chumote, custom stream CGIs) — all via `/tmp/flashplayer.event` + HUP |

## 3. Zurk's impersonation architecture (the prior art for M2)

Mechanism, end to end:

1. **DNS capture:** `/psp/hosts` maps every chumby host (`xml.`, `update.`,
   `widgets.`, `files.`, `music.`, `bor.`… ) **plus `tuner.pandora.com`** to
   `127.0.0.1` (plus an MVPS ad-block list).
2. **Local lighty** (`document-root /mnt/usb/lighty/html`) serves the
   impersonated endpoints, mixing three techniques:
   - **Static XML stubs**: `/xml/authorize` → `<chumby id="offline-ID">
     <name>offline-name</name></chumby>`; `/xml/chumbies` (index.html) →
     `<chumby id="Zurk-Chumby"><name/><profile href="/xml/profiles" …/>
     <user username="Zurk"/></chumby>`; `/xml/skins/…`.
   - **CGI shell scripts** via `url.redirect`: `/xml/controlpanel` →
     `controlpanel.sh` (emits the download_cp XML envelope: url
     `http://localhost/xml/c1/controlpanel.swf`, md5, `<location>/tmp`,
     `<parameters>` incl. `defaultUpdateTime=9999`,
     `defaultProfileTime=9999` — i.e. **server-pushed panel parameters
     exist**); `/xml/profiles` + `/xml/setprofile` → `profiles.sh`
     (multi-channel: offline-mode / Squeezebox / Kitchen Timer);
     `/music_sources/show` → `musicsourcesshow.sh`; `/update` → `update.sh`
     ("no update"); `/chumcast/show` → `tune.sh`; photo/clock/etc helpers.
   - **Selective real-network passthrough**: `/shoutcast/show` redirects to
     the *real* `yp.shoutcast.com/sbin/tunein-station.pls…&k=<api key>` —
     i.e. zurk kept genuine internet radio alive while mocking everything
     chumby-account-related. This is exactly our `mock-forever` vs
     `real-network-later` split, implemented.
3. **Widget delivery:** profile XML references widgets with
   `file:////mnt/usb/www/…` hrefs for thumbnails/movies — **the panel
   accepts `file://` widget URLs**, so zurk serves widget SWFs from local
   disk without any widget-server emulation. (Profiles also confirm the
   exact `<widget_instance>` XML shape our fixtures need: name, description,
   version, `<mode mode="timeout" time="180"/>`, `<access sendable=…
   deleteable=…/>`, user, thumbnail/movie hrefs.)
4. **Panel binary:** stock(-looking) 2.8.84 dropped in place; downloads
   neutralized by the impersonated `/xml/controlpanel` envelope pointing at
   the local copy.

## 4. Takeaways for our M2 design (chumby-host.md inputs)

- The 2.8.x panel runs fine against an environment that is ~10 static XML
  files + ~10 tiny CGI responses. Zurk's stub set is effectively a
  **field-tested minimal fixture corpus** — we can lift formats (and several
  literal files) directly into `fixtures/`.
- `defaultUpdateTime/defaultProfileTime=9999` parameters are the documented
  way to quiet the update/profile polling loops — better than stubbing timers.
- `file://` widget hrefs mean M2 can do real widget playback from local disk
  with **no** widget-server fixture, on either the slave or localCache path.
- We don't need DNS tricks: unlike zurk (who couldn't touch the player),
  we own the player — URL interception in Ruffle replaces hosts+lighty.
- Multi-channel via profiles.sh shows channel switching works fully offline —
  relevant to question D2 in 05-screens.md.

## 5. Explicitly not done (and why)

- Script-level diff shipped-1.7.2-builtin vs 2.8.87b3 vs zurk-2.8.84 SWFs:
  descoped by user at CHECKPOINT 1. Revisit only if a 2.8.87b3 behavior
  needs explaining via history (e.g. the "old firmware, forcing wireless"
  branch seen in run 7).
- Verification that zurk's 2.8.84 is bit-identical to Chumby's official
  release: no independent reference copy found within the time box.
