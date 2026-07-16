# Open issues

One block per issue: Number, Timestamp, Title, Status, Description.
Appliance-side issues only; player issues live in ruffle/claude/issues.md.

---

Number: 1
Timestamp: 2026-07-17, 00:15
Title: Check chumby_accel.c applicability to the Pi kernel.
Status: open
Description: The chumby kernel source is public (GPL), so the accelerometer
driver behind /dev/accel — chumby_accel.c, "2.1-Kionix-Ironforge", bunnie's
one-off that also carries the dcid EEPROM driver — can be read and possibly
ported. Assess what a port to the Raspberry Pi kernel would take: it
bit-bangs the KXP74 through i.MX21 GPIO calls (imx_gpio_mode/imx_gpio_write,
own spi_exchange_data), so the SPI layer would need rewriting against the
Pi's spidev/SPI subsystem, keeping the in-kernel averaging/impact logic and
the 56-byte accelReadData read() contract. Compare against the current
preference — a userspace spidev reader in PiHost feeding ASnative(5,60)/
(5,61) — which needs no kernel module; the port only pays off if something
besides PiHost must consume /dev/accel. Findings and the full access chain:
hardware/chumby-hat/accelerometer.md.

---

Number: 2
Timestamp: 2026-07-17, 01:00
Title: Daughtercard breakout board (supersedes the HAT concept).
Status: open
Description: Reframe hardware/chumby-hat/ from a Pi HAT to a passive breakout
with a soldered-on 2x13 header the chumbilical plugs into. Mapping: DC jack →
USB-A power-only port (decide: 5 V supply into the barrel jack, or buck for
the original 12 V wart); speakers → 4-pin PH2.0 (Waveshare amp); 2x USB-A
male toward a hub carrying DATA + GND ONLY — VBUS for both jacks is the
shared P50V net, fed once from the breakout's own 5 V rail, never from the
hub's ports, so both USB ports work without back-feeding the hub; headphones
→ screw terminal; battery unconnected. Everything else on a single 2x5
dupont block mating Pi header pins 17-26 (3V3, SPI0 with both CEs, 2x GND,
GPIO24/25 for bend/reset — moves bend off FR3's GPIO17, one gpio-key config
line). Open before drawing: confirm shared P50V and the power switch sitting
in series with RAW_PWR (no button net crosses the chumbilical); HP_NOTIN
placement (screw terminal, dupont, or dropped); measure the cable-end
housing (dimensions, polarization, latches) to pick the through-hole header;
mechanical alignment of rigid USB-A plugs with a hub — fallback is a short
captive cable. Alternative still on the table: replace the connectors on the
daughtercard itself (unassessed).
