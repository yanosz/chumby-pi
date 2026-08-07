# Pins in use — the wired prototype

What is actually connected today between the chumby classic's
daughtercard (via the chumbilical cable) and the Raspberry Pi, plus
which daughtercard connectors are wired but terminate somewhere other
than the Pi.

This is the *as-built* record. The full 26-pin connector table, the two
competing numbering schemes and the per-pin provenance live in
[`accelerometer.md` §3](accelerometer.md) — read that first if you need
a pin this page does not list. **That file lives only on the
`hardware-ideas` branch**, along with the rest of `hardware/`; this page
is the one piece carried on `dev` as well, so on `dev` the links to it do
not resolve.

**All breakout numbers here are physical** (pstrick2's convention: pin 1
left-front with the polarization notch toward the viewer, front row
1–13, back row 14–26).

Wire colours are Jan's harness, not a board marking — they identify a
wire in the bundle and mean nothing to anyone rebuilding it from
scratch.

## The two pictures

**Raspberry Pi 40-pin header** — the pin numbers in the "Pi pin" column
below are the physical numbers in this diagram:

![Raspberry Pi GPIO pinout](https://www.raspberrypi.com/documentation/computers/images/GPIO-Pinout-Diagram-2.png)

<sub>Source: [Raspberry Pi documentation, Raspberry Pi hardware — GPIO](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#gpio-and-the-40-pin-header).</sub>

**Ironforge daughtercard, with pstrick2's continuity annotations** — the
breakout numbers below are read off these two photos. Top side carries
the chumbilical numbering key itself (`14.15…26` / `01.02…13` printed
across the connector) and the connector-by-connector labels; bottom side
carries the headphone jack and the reset switch:

![Daughtercard, top side, pin numbers annotated](https://i.imgur.com/Y1xdBqo.png)

![Daughtercard, bottom side, jack and switch annotated](https://i.imgur.com/6JYD1EX.png)

<sub>Source: pstrick2, [chumby forum post #58250](https://forum.chumby.com/viewtopic.php?pid=58250#p58250), 2019-12-01. Hotlinked, not copied into this repo.</sub>

Note what the top photo settles on its own: the two ground pins are
labelled at the DC jack as `15,02 G`, and `+5V` appears at both USB
connectors — matching §3's table without needing the schematic.

## Wired to the Pi

| Function | Breakout | Wire | Pi pin | Pi signal | Net (§3) |
|---|---:|---|---:|---|---|
| Reset / power button | 5 | yellow | 5 | GPIO3 | `CHUMBY_RESET_REQ` |
| Reset / power button | 6 | green | 6 | Ground | `P33VBKUP` |
| Bend sensor | 9 | purple | 29 | GPIO5 | `CHUMBY_BEND` |
| Ground (bend return) | 2 | grey | 30 | Ground | `GND` |

Both grounds are deliberately the Pi header pin *adjacent* to their
signal, so each pair is one two-wire run: 5/6 and 29/30.

**Reset / power button.** `dtoverlay=gpio-shutdown` with its defaults
(GPIO3, active-low, pull-up) is exactly this wiring; a halted Pi wakes
on a GPIO3 short with no configuration at all. Breakout 6 is
`P33VBKUP` on the daughtercard — the reset switch's high side, not a
ground — and is nevertheless wired to Pi ground here: the switch just
closes 5 to 6, and the mainboard's active-high biasing was the
mainboard's affair. See issue 3 in `claude/issues.md` for how this was
tracked down, and §3 for the consequence: **pin 6 cannot stay wired to
ground once the accelerometer goes in**, if `P33VBKUP` also turns out to
supply the KXP74 and the ID EEPROM.

**Bend sensor.** `dtoverlay=gpio-key,gpio=5,keycode=102,label=chumby-bend`
makes the button a keyboard-class evdev device that cage forwards to the
player as KEY_HOME; no code was needed on the player side. The switch
returns to daughtercard ground (`09 / G` on its own connector in the top
photo), which is why breakout 2 works as its return.

Pin choice is per-device and not yet settled across the project: FR3
says GPIO17, issue 2's breakout proposes GPIO24/25, this harness uses
GPIO5.

## Wired, but not to the Pi

| Function | Breakout | Wire | Net (§3) | Other end |
|---|---:|---|---|---|
| Headphone jack | 10 | blue (shield) | `HP_NOTIN` | display, 3.5 mm audio |
| Headphone jack | 11 | green | jack contact (return?) | display, 3.5 mm audio |
| Headphone jack | 24 | *unrecorded* | `HPRIGHT` | display, 3.5 mm audio |
| Speaker, left | 12 | yellow | `SPKL_VO2` | display, speaker interface |
| Speaker, left | 13 | white | `SPKL_VO1` | display, speaker interface |
| Speaker, right | 25 | red | `SPKR_VO2` | display, speaker interface |
| Speaker, right | 26 | black | `SPKR_VO1` | display, speaker interface |
| DC barrel jack | 14 | red | `RAW_PWR` (12 V) | 12 V supply |

### Where the audio comes from

Both audio runs originate at **the display**, not at the Pi: the
Waveshare 3.5" HDMI LCD (E) carries "3.5mm audio and speaker interface,
support HDMI audio output" (vendor wiki), and those two outputs feed the
daughtercard's headphone jack and its two speaker pairs respectively.
Audio therefore travels HDMI → display board → chumbilical → chumby
hardware, and never touches the Pi's own 3.5 mm jack.

This **supersedes design.md §7's plan**, which assumed headphones would
hang off the Pi's jack with `HP_NOTIN`→GPIO driving a software sink
switch. With the display as the source, a Pi-side sink switch cannot mute
anything: the mute would have to happen on the display's speaker feed.

**One HDMI sink, not two** (measured 2026-08-06, second test Pi
192.168.210.159). The Pi cannot address the display's jack and its
speaker interface separately: there is exactly one sink,
`alsa_output.platform-3f902000.hdmi.hdmi-stereo` (card 1 `vc4hdmi`), and
the split happens on the display board downstream of HDMI. So any
speaker/headphone switching is the board's own doing — PipeWire has
nothing to route.

**FIXED: the player's audio never reached the display at all.** The
PipeWire default sink was `Built-in Audio Stereo` — the *Pi's own* analog
3.5 mm jack — and `ruffle_desktop` was streaming into `bcm2835
Headphones`. HDMI carried no audio, so neither the display's speakers nor
its jack could ever have made a sound, whatever the board does. Switching
the default to the HDMI sink moved the player's stream to `MAI PCM
i2s-hifi-0` and produced audible tone on the chumby hardware (Jan at the
device, 2026-08-06). The switch is WirePlumber runtime state and survived
a reboot; **it is not packaged** — the appliance should select the HDMI
sink deliberately rather than inherit PipeWire's default pick, and
`CHUMBY_AUDIO_DEVICE` does not cover this (it steers mpv only, not the
player's `cpal` output). Open packaging question.

The display end is confirmed willing: its ELD advertises
`monitor_name WS-35-640`, LPCM stereo, 32/44.1/48 kHz, FL/FR, and the Pi
transmits `IEC958_SUBFRAME_LE` at 48 kHz with the PCM `RUNNING`.

### Headphone detection: the display has it, in hardware

**ANSWERED 2026-08-06 — the (E) mutes its speaker interface whenever a
plug sits in its own 3.5 mm jack.** Vendor documentation could not settle
this (the wiki does not mention it and Waveshare publishes no schematic
for this board, only 3D drawings), so it was measured at the device: a
440 Hz tone on the HDMI sink, `RUNNING` PCM, everything on the Pi side
held constant, with the plug as the only variable. Jack out → tone at the
speakers. Jack in → silence. Jan at the device both times.

An earlier apparent version of this result was **discarded as a
confound** and is recorded so nobody re-derives it: audio first became
audible in the very step that the jack came out, but both preceding
silent observations had no signal flowing at all (first the wrong default
sink, then a `speaker-test` that had already exited on SSH close), so the
jack-in case had never actually been tested. Only the controlled repeat
counts.

**The consequence for this harness:** it feeds the daughtercard's
headphone jack *by plugging into the display's jack*, and that plug is
exactly what mutes the speaker interface. As wired, speakers and
headphones are mutually exclusive — the tap itself silences the speakers.
Tapping the display's jack at its solder pads instead would keep the
detect switch closed and both outputs live; which way to go belongs to
issue 2.

`HP_NOTIN` (breakout 10) stays wired and consumed by nothing. It cannot
mute anything by itself here: the speakers are fed by the display's
amplified output, which no Pi GPIO reaches.

Unconfirmed in the table above, both flagged rather than guessed away:

- the speaker colour-to-pin assignment is taken positionally from how
  the four were listed (12 yellow, 13 white, 25 red, 26 black); §3
  already notes that L/R and VO1/VO2 *inside* each pair is pstrick2's
  photo label and unverified;
- the headphone harness has three wires and two recorded colours (blue =
  shield, green), so pin 24's colour is simply not on record.

### The 10/23 question

The bottom photo annotates **two** of the headphone jack's four lugs
with the same callout, `23,10` — so breakout 10 and 23 beep out to the
same contact, which is why this harness uses 10 and leaves 23 unwired.
Two readings, and they differ in consequence:

1. one contact with two solder points → 10 and 23 are the same net,
   always, and using either is equivalent;
2. the jack's **normalling switch** → 10 (`HP_NOTIN`, jack-detect) sits
   on the tip's switch contact and is closed to 23 (`HPLEFT`) only while
   no plug is inserted. Audio fed into 10 would then vanish the instant
   headphones are plugged in, and 23 is the pin actually wanted for the
   left channel.

Reading 2 fits the recorded net names and is the classic switched-jack
arrangement, but the photo cannot distinguish them. **Open, and cheap to
close:** meter 10 against 23 with a plug inserted. Until then this is
inference, not a finding.

## Not wired

Everything else on the cable, for completeness — see §3 for the nets:

| Breakout | Net | Why not |
|---:|---|---|
| 1 | `BATTERY` (+) | battery deliberately unconnected (issue 2) |
| 3, 4, 16, 17, 18 | `CSPI1` MISO/MOSI/SCLK/SS0/SS1 | the accelerometer + ID EEPROM path — planned, issue 1 |
| 7, 8, 20, 21 | `USBB_DN/DP`, `USBB2_DN/DP` | USB data, issue 2's hub plan |
| 15 | `GND` | second ground; 2 is enough for the two switches |
| 19 | unknown | not identified by any source |
| 22 | `P50V` (USB VBUS) | issue 2 feeds this locally, not from the cable |
| 23 | `HPLEFT` | see the 10/23 question above |

## Provenance

- Wiring, wire colours and pin choices: Jan's harness, recorded
  2026-08-06.
- Breakout pin numbering and the daughtercard-side nets: pstrick2's 2019
  continuity survey plus Jan's 2026-07-19 meter readings, as reconciled
  in [`accelerometer.md` §3](accelerometer.md) — that page is the
  authority, this one only says what is plugged in.
- Pi pin/GPIO correspondence: the Raspberry Pi documentation diagram
  above.
