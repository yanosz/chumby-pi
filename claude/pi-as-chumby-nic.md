# Pi Zero W as the chumby's NIC

The chumby's own wifi is out, so a Pi Zero W fronts it: house wifi on one
side, USB-gadget ethernet on the other, routed and NAT'd between them. The
chumby sees an ordinary USB ethernet adapter and takes DHCP from the Pi.

Status: working. Link, naming, DHCP, NAT and time sync all survive a reboot
of either box. The Pi's gadget side is deliberately **not** NetworkManager —
see "Why the gadget left NetworkManager".

## Two mechanisms behind almost everything here

**1. The host names the interface from the MAC.** `usbnet` picks the name:

```c
strcpy(net->name, "usb%d");
/* heuristic: "usb%d" for links we know are two-host, else "eth%d"
 * when there's reasonable doubt. userspace can rename the link... */
if ((dev->driver_info->flags & FLAG_ETHER) != 0 &&
        (net->dev_addr[0] & 0x02) == 0)
    strcpy(net->name, "eth%d");
```

`cdc_ether` sets `FLAG_ETHER`, so the deciding test is bit `0x02` of the
first MAC byte. `g_ether` defaults to random **locally administered** MACs,
which sets that bit and forces `usb0` — and nothing on the chumby can use a
`usb*` interface:

- `add.sh` selects with `ifconfig -a | awk '/^eth/ {print $1;}'`.
- `network_interface` — which `network_status.sh` and the panel's readiness
  check both go through — only ever considers `eth*` or `rausb*`, picked by
  `type=` in `/psp/network_config`, and takes the first match.

Pinning a host MAC with that bit clear fixes all of it at once.

**2. mountmon derives the module name from the moddef FILENAME.** `add.sh`
greps `vendor:product` against `/etc/mountmon/usbnet-*.moddef`, then loads
`/drivers/<suffix>` for each file that matched. Stock ships only
`usbnet-asix.moddef` and `usbnet-pegasus.moddef`, so `cdc_ether.ko` — present
in `/drivers` all along — is reachable only through a file named
`usbnet-cdc_ether.moddef`. Once loaded, `cdc_ether` binds by CDC-ECM *class*,
so the ID list only has to get the device recognised.

## What was needed on the Pi

Raspberry Pi OS (kernel 6.18). `g_ether` is loaded from
`/etc/modules-load.d/usb-gadget.conf`, i.e. via modprobe, which is why
`/etc/modprobe.d` options apply. NetworkManager runs a `USB Gadget (shared)`
profile on `usb0` with 10.12.194.1/28 and its own dnsmasq serving .3–.14;
stock `rpi-usb-gadget-ics.service` (a compiled libnm binary, `ics-watch`)
arpings 192.168.137.1 to decide between client and shared profiles.

| Change | Why |
|---|---|
| **`/etc/modprobe.d/usb-gadget-mac.conf`**: `options g_ether dev_addr=b8:27:eb:5c:f7:9d host_addr=b8:27:eb:5c:f7:9e` | Clears MAC bit `0x02`, so the chumby names the link `eth<n>` and the whole vendor path can see it. `b8:27:eb` is the Raspberry Pi OUI, adjacent to this box's `wlan0` (`…:9c`). |
| **`/etc/systemd/network/10-usb0.network`** | Owns the gadget link: `Address=10.12.194.1/28`, `IPMasquerade=ipv4` (networkd writes the nftables NAT itself), `IPv4Forwarding=yes`, `DHCPServer=yes` with `EmitDNS=yes` / `DNS=8.8.8.8`, and `RequiredForOnline=no` so boot never blocks on the chumby being plugged in. |
| **`/etc/NetworkManager/conf.d/99-unmanaged-usb0.conf`** | `unmanaged-devices=interface-name:usb0`, so NM keeps its hands off. NM still owns `wlan0`. |
| **`rpi-usb-gadget-ics.service` disabled** | Stock libnm binary that flips usb0 between client and shared NM profiles by arping 192.168.137.1. Meaningless once NM does not own usb0, and it was a plausible aggravator of the NM bug. |
| **`systemd-networkd` enabled**, `systemd-networkd-wait-online` **disabled** | Enabling networkd pulls wait-online in, where it fails with "Timeout occurred while waiting for network connectivity" — correctly, since the only managed link is `RequiredForOnline=no`. `NetworkManager-wait-online` covers the real uplink. |

networkd replaces three NM jobs at once: the address, dnsmasq (its own DHCP
server), and the masquerade (`table ip io.systemd.nat`, with
`elements = { 10.12.194.0/28 }` in the `masq_saddr` set). A side effect worth
knowing: without NM's cloned-MAC override, the Pi-side netdev now actually
uses the pinned `dev_addr`.

Nothing else on the Pi was touched. In particular `config.txt` is left as
rpi-imager wrote it, including both `dtoverlay=dwc2,dr_mode=host` and
`dr_mode=peripheral`; peripheral wins in practice (a UDC registers, hosts
enumerate). Note NM applies a *cloned* MAC to the Pi-side netdev, so
`dev_addr` never shows on `usb0` — harmless, only `host_addr` decides naming.

Consequence of peripheral mode: the OTG port is the Zero W's only USB, so the
box cannot host a keyboard while it is the NIC, and its HDMI console is
read-only.

## What was needed on the chumby

Root is **cramfs, read-only**; `/psp` is jffs2 and writable. `rcS` offers
three hook points and the differences matter:

| Hook | rcS line | How | Relative to |
|---|---|---|---|
| `userhook0` | ~42 | **sourced** — an `exit` kills the boot | before mountmon (69) and `start_network` (~199) |
| `userhook1` | ~209 | executed | after `start_network` |
| `userhook2` | ~311 | executed | late |

Never create `/psp/rfs1/rcS`: rcS runs it and then `exit`s, replacing the
whole boot.

**`/psp/rfs1/userhook0`** — two jobs:

1. Copies `/etc/mountmon` to tmpfs, adds `usbnet-cdc_ether.moddef`
   (`2e8a:0013`, the ID Pi OS actually presents, plus the legacy `g_ether`
   IDs as fallback), and bind-mounts it over the read-only original.
2. Waits for a NIC **only when none is on the bus yet**. A dongle is
   enumerated before this point, so a dongle-only boot waits zero seconds;
   the Pi's gadget does not answer on the bus until its own kernel has loaded
   `g_ether` (~2 min), and everything downstream gets one shot at it. The
   condition is "is any id from the moddef files present" — 99 ids, including
   both the dongle's and the Pi's. Capped at 240s so a missing NIC cannot
   hang the boot.

**`/psp/rfs1/userhook1`** — starts `sshd` (host keys and `sshd_config` ship
in `/usr/local/etc` on the read-only root, so nothing is generated; nothing
in `rcS` starts it), runs `start_network`, finds the gadget **by its pinned
MAC** (the name varies: `eth1` beside a dongle, `eth0` alone), brings it up,
and — only if nothing else holds a default route — asks for a lease and waits
for it. Then `sync_time.sh -b`.

Why each part is there:

- The bring-up: `add.sh` skips it whenever another `eth*` is already RUNNING.
  It is also the altsetting trigger — an ECM gadget asserts carrier only once
  the host selects the data altsetting.
- The lease wait: `add.sh` runs `udhcpc -R -n` exactly once, and `-n` exits
  the instant no server answers.
- The default-route guard: so a temporarily attached dongle keeps its route
  instead of having it stolen mid-session.
- The time sync: `rcS` syncs only when the internal wifi has carrier
  (`running=\`ifconfig rausb0 |grep RUNNING\``), which never passes here.

`/psp/rfs1/userhook2` is a pre-existing control-panel cache hook, untouched.

## Verified

- `cdc_ether` loads from the moddef; the chumby recognises the gadget.
- After the MAC change the link is `eth1` with `HWaddr B8:27:EB:5C:F7:9E`.
- Lease obtained from the Pi (dnsmasq under NM at the time; networkd's own
  DHCP server since).
- With NAT present: `ping 8.8.8.8` 0% loss, and
  `http://www.chumby.com/crossdomain.xml` fetches in milliseconds.
- sshd survives reboot from `userhook1`; `sync_time.sh` sets the clock.
- Dongle-only boot does not wait (its id `0b95:772a` is among the moddef ids).
- **After a Pi reboot, unaided:** `table ip io.systemd.nat` present, `usb0`
  `routable (configured)`, NM `unmanaged`. Before that reboot the chumby had
  `nameserver 8.8.8.8`, a default route via 10.12.194.1, and fetched
  `chumby.com/crossdomain.xml` at 1.23 MB/s.

## Why the gadget left NetworkManager

NM's shared mode reported activation success while silently skipping its
nftables stage. Reproduced twice from cold boot: `nmcli` said
`USB Gadget (shared)` was `connected`, the address was assigned and dnsmasq
was running, but `nft list tables` was **empty** — no masquerade. `nmcli con
down/up` on the profile then created `table ip nm-shared-usb0` and everything
worked. NM logged one clean "Activation: successful" and nothing about the
firewall, while `nft_chain_nat` was loaded — so a nat chain had existed at
some point in the boot.

This is not local misconfiguration and not Raspberry-Pi-specific. The same
failure — shared mode reporting success with its firewall/routing setup
missing, cured by re-activating — is reported across distros and years:

| Report | When | Platform |
|---|---|---|
| [NM #1016](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1016) | 2022 | Ubuntu 22.04: hotspot auto-starts at boot, clients get no internet, fixed by re-activating it |
| [NM #1390](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1390) | 2023 | "About 1 out of 10 times … NetworkManager will just fail to set up the routing for the hotspot" |
| [NM #1827](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1827) | 2025 | Ubuntu, NM 1.46: hotspot startup does not configure the firewall for sharing |
| [NM #1935](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1935) | 2026 | Mobian, NM 1.56: shared connection stops forwarding |
| [Pi forums t=366434](https://forums.raspberrypi.com/viewtopic.php?t=366434) | 2024 | Pi OS, shared `usb0`: no tables at boot, `nmcli con up` yields `table ip nm-shared-usb0` |
| [trixie-feedback #62](https://github.com/raspberrypi/trixie-feedback/issues/62) | 2026 | Pi OS Trixie: empty ruleset at boot, no error logged, cured by restarting NM |
| [Arch forums](https://bbs.archlinux.org/viewtopic.php?id=269863) | — | NM's nftables backend omitting the dnsmasq input rule |

Every upstream issue above is `unassigned` / `workflow::triage` and was closed
roughly a year after filing, which reads as staleness rather than a fix — and
NM **1.52.1** here still does it. Maintainer replies could not be read: the
GitLab notes API needs authentication and the web pages sit behind Anubis bot
protection, so only the reports themselves were verifiable.

Hence networkd for this link: it applies a static file at link-up, with no
D-Bus activation, no auto-activation race and no profile switcher. One
NM-adjacent variant was checked and ruled out — per-interface forwarding
(NM #1935) is `1` on `all`, `wlan0` and `usb0` here.

**Not done: moving `wlan0` off NM too.** It would finish the job, but wifi is
the only remote path to the Pi — the OTG port belongs to the gadget, so there
is no console keyboard and HDMI is read-only. A bad
`wpa_supplicant-wlan0.conf` means recovery by pulling the SD card. Worth
doing at the bench, not remotely.

## Dead end: OpenWrt

An OpenWrt 25.12.5 image for `bcm27xx/bcm2708` was built and flashed (harness
in `openwrt/`, credentials never in the build, LuCI over the gadget as the
config path). LAN and LuCI worked, but it was **unstable on this Broadcom
hardware**, so the box went back to Pi OS. Kept as a record; nothing depends
on it. Findings still worth knowing: `kmod-usb-gadget-eth` autoloads only
`usb_f_ecm`, not the gadget; `board.d/02_network` makes wlan0 the LAN on a
Zero W unless both `/etc/config/network` and `/etc/config/system` are
shipped; and `config.txt` must be patched in the Image Builder tree because
`FILES=` cannot reach the FAT partition.

## Security note

The house wifi PSK was exposed during this work (read off the Pi and sent to
the model API). It needs rotating; until then the wifi side cannot be treated
as a trust boundary.

## Open

- **A Pi reboot orphans the chumby** until the chumby also reboots: the
  gadget re-enumerates, the chumby's netdev is destroyed and recreated, and
  its DHCP client dies with it. Seen as `RX: 0 bytes, 0 packets` on the Pi's
  `usb0` with carrier up. claude/issues.md #7.
- **The panel blocks forever on a no-timeout `wget`** to chumby.com whenever
  its route leads to a blackhole — this is what "the chumby hangs on boot"
  actually was. claude/issues.md #8.
- End-to-end from cold boot of *both* boxes not yet observed; that run is
  what tests `userhook0`'s conditional wait against a Pi that is already up.
- `/psp/network_config` has `type="lan"`, which is what makes
  `network_interface` look for `eth*`. Worth confirming it stays that way.
