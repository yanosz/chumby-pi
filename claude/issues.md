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
