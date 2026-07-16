# Accelerometer report — Kionix KXP74 on the Ironforge daughtercard

Findings from the 2026-07-17 investigation: where the chumby classic's
accelerometer lives, how the stock software reads it, and what it takes
to hook it to the Raspberry Pi. Sources: the Ironforge mainboard
schematic (`pdf/IRONFORGE_MX21_V1_8_FINAL.pdf`, sheet 5), the firmware
backup (`/home/jan/chumby_backup`), the strings of the stock
`chumbyflashplayer.x`, the chumby wiki, and chumbysphere forum threads
[#3193](https://forum.chumby.com/viewtopic.php?id=3193) /
[#2291](http://forum.chumby.com/viewtopic.php?id=2291).

## 1. The part and where it sits

**Kionix KXP74-1050**, 3-axis, 12-bit, SPI-only (2.7–5.25 V supply;
wiki spec list names the part). It is **on the daughtercard**, not the
mainboard:

- The mainboard schematic contains no Kionix part; instead sheet 5
  routes the whole **CSPI1** bus (MISO/MOSI/SCLK/**SS0/SS1**) to the
  chumbilical connector P701.
- Forum #3193: *"the 'daughtercard' … holds the accelerometer, the
  power, headphone and usb connectors, and the power switch."*
- Forum #2291: with the daughtercard unplugged the accel driver hangs
  in `spi_exchange_data` — the SPI slave left with the card.

Two chip selects cross the cable because the card carries **two** SPI
devices: the accelerometer and an **Atmel AT25080A** ID EEPROM (the
`dcid` device — daughtercard serial/ID). Which select belongs to which
chip is not determinable from the mainboard side (no daughtercard
schematic exists) — it must be probed or beeped out.

## 2. How the stock software reads it

The full chain, bottom to top:

1. **Kernel:** `chumby_accel.ko` (version `2.1-Kionix-Ironforge`,
   bunnie's GPL driver; the same module carries the `dcid` EEPROM
   driver, `1.0-Atmel-25080A-Ironforge`). `etc/init.d/rcS` insmods it
   and mknods **`/dev/accel`** (char device, dynamic major). A
   human-readable `/proc` entry exists too (`accel_read_proc`).
2. **Read protocol:** plain `read()` of one fixed 56-byte record
   (14 × u32), no ioctls — documented on the chumby wiki
   (VolumeChangeWithAccelerometer):

   ```c
   struct accelReadData {
       unsigned int version;
       unsigned int timestamp;
       unsigned int inst[3];    /* x, y, z raw counts */
       unsigned int avg[3];
       unsigned int impact[3];
       unsigned int impactTime;
       unsigned int impactHint;
       unsigned int gRange;
   };
   ```

   Values are unsigned centered raw counts; 2048 = level (12-bit
   mid-scale).
3. **Flash player:** `chumbyflashplayer.x` has a `SensorObject` that
   opens `/dev/accel`, `select()`s and reads that struct each pump, and
   exposes it to ActionScript as `getaccelvalue` in the same native
   table as `getbend`/`getrawx`/`getlight`.
4. **Panel:** calls **ASnative(5,60)** `_accelerometer` and **(5,61)**
   `_accelerometerSigned` (`ruffle/core/src/chumby/avm.rs`). Observed
   protocol (recorded at `ruffle/core/src/chumby/fixture.rs`): arg `0`
   is a driver-version probe (answer 1); the intro's ball page polls
   args **5 and 6** for axes. Our fixture answers a constant 2048.

   *Unverified hypothesis:* if the arg indexes the 14-word struct,
   args 5/6 are `avg[0]`/`avg[1]` (averaged x/y) and arg 0 hits
   `version`. Confirming needs bunnie's GPL kernel source or the
   decompiled panel's call sites.

Everything above the wire is ours to replace: on the Pi, PiHost would
read the KXP74 over `spidev` and answer (5,60)/(5,61) from live values
in place of the static fixture — same live-derivation principle as the
status page.

## 3. Wiring it to the Pi

The Pi 3B+ header has SPI0 at 3.3 V, matching the i.MX21's I/O levels —
a straight wire-up:

| Chumbilical pin | Net | Pi header |
|---:|---|---|
| 5 | `CSPI1_MOSI` | GPIO10 / pin 19 (MOSI) |
| 3 | `CSPI1_MISO` | GPIO9 / pin 21 (MISO) |
| 4 | `CSPI1_SCLK` | GPIO11 / pin 23 (SCLK) |
| 6 | `CSPI1_SS0` | GPIO8 / pin 24 (CE0) |
| 8 | `CSPI1_SS1` | GPIO7 / pin 26 (CE1) |

Power: the only 3.3 V net crossing the chumbilical is `P33VBKUP`
(pin 9), so presumably it feeds the KXP74 + EEPROM and would come from
the Pi's 3.3 V rail — **inference from the net list, verify before
relying on it** (KXP74 tolerates 2.7–5.25 V, the AT25080A 1.8–5.5 V, so
the rail itself is safe either way; the SPI signal levels must stay
3.3 V).

## 4. Q: connector pitch — do jumper wires fit?

**Yes — it is a standard 2.54 mm (0.100") grid.** The mainboard-side
P701 is a **Molex 71349-2011**, from Molex's **C-Grid** family:
shrouded dual-row headers on 2.54 mm pitch with 0.64 mm (0.025")
square posts — the same post size and grid as ordinary 0.1" pin
headers and Dupont jumper housings.

For prototyping that means:

- The **cable's receptacle** (which mated to that header) accepts
  0.64 mm square pins on a 2.54 mm grid → standard **male jumper
  wires plug straight in**, or a bare 2×13 0.64 mm pin header can
  serve as a mating stub.
- The HAT's `P1` can simply be a standard 2×13 0.1" shrouded box
  header, subject to the existing verify item: identify the exact
  housing the cable actually ends in before ordering (polarization
  bump/slot orientation).

## 5. Q: is there a Pi/Linux driver for the KXP74-1050?

**No ready-made one.** Checked 2026-07-17:

- Mainline Linux has no KXP74 driver. The kernel's Kionix drivers —
  IIO `kxsd9` (SPI+I2C), `kxcjk-1013`, `kx022a` (SPI), input `kxtj9` —
  target newer parts with different register maps; none claims KXP74
  compatibility.
- ROHM (Kionix's owner) maintains out-of-tree drivers on
  [GitHub](https://github.com/RohmSemiconductor/Linux-Kernel-Sensor-Drivers)
  for current parts only; the KXP74 (2005) is not among them.

Options, best first:

1. **Userspace over `spidev`** — enable SPI0 (`dtparam=spi=on`), read
   the KXP74's registers directly from PiHost in Rust. The part is an
   SPI register device with an internal 12-bit ADC; no interrupt or
   kernel machinery is needed for the panel's polling use. Fits the
   project's Rust-over-shell principle, and the values terminate in
   `fixture.rs` anyway.
2. **Port bunnie's GPL `chumby_accel.c`** from the chumby kernel drop
   as an out-of-tree module reproducing `/dev/accel` — only worth it
   if something other than PiHost must consume the data.

## Open verification items (adds to the README's list)

1. Which chip select (SS0/SS1) is the accelerometer vs. the AT25080A.
2. Which chumbilical pin actually powers the two SPI chips
   (presumed `P33VBKUP`, pin 9).
3. Confirm the cable-end housing is a C-Grid-compatible 2×13
   receptacle before sourcing `P1`.

Sources: [Molex C-Grid 2.54 mm catalog](https://ptelectronics.ru/wp-content/uploads/katalog/Molex/molex_pcb_wire_connectors_2,54mm_pitch.pdf),
[SnapEDA 71349 series](https://www.snapeda.com/parts/71349-1001/Molex/view-part/),
[ROHM/Kionix Linux drivers](https://github.com/RohmSemiconductor/Linux-Kernel-Sensor-Drivers),
[chumby wiki: Hacking hardware](http://wiki.chumby.com/index.php?title=Hacking_hardware_for_chumby),
[chumby wiki: VolumeChangeWithAccelerometer](http://wiki.chumby.com/index.php?title=VolumeChangeWithAccelerometer).
