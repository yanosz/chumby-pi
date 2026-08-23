# Pi as a real chumby's NIC

A chumby whose own wifi has died can still get online: a Raspberry Pi with
USB-OTG becomes its network card. The Pi joins wifi, presents a USB ethernet
gadget to the chumby, and NATs between the two. The chumby sees an ordinary
USB ethernet adapter.

This is the opposite of the rest of this repo, which turns a Pi *into* a
chumby (see `docs/setup.md`). Here the chumby stays the chumby.

Verified on a Pi Zero W with Raspberry Pi OS trixie against a chumby running
2.6.16-csb.

## Contents

| File | Runs on | Purpose |
|---|---|---|
| `init_pi.sh` | the Pi, as root | Pins the gadget MACs, installs the NAT |
| `init_chumby.sh` | the chumby, as root | Copies the hooks into `/psp/rfs1/` |
| `userhook0` | the chumby, at boot | Teaches mountmon the gadget, waits for it |
| `userhook1` | the chumby, at boot | Brings the link up, DHCP, clock |

## Quick start

**Pi** — flash Raspberry Pi OS with gadget mode enabled (rpi-imager advanced
options), wifi configured, and your ssh key installed. Then:

```
sudo ./init_pi.sh
```

**Chumby** — copy this folder over (scp, or a USB stick — see below) and:

```
./init_chumby.sh
reboot
```

That's it. Re-run `init_pi.sh` after any reflash of the Pi; the chumby side
lives in `/psp` and survives.

## The two mechanisms behind all of it

Neither is guessable, and everything else follows from them.

**1. The chumby names the interface from its MAC.** `usbnet` picks the name:

```c
strcpy(net->name, "usb%d");
/* heuristic: "usb%d" for links we know are two-host, else "eth%d"
 * when there's reasonable doubt. userspace can rename the link... */
if ((dev->driver_info->flags & FLAG_ETHER) != 0 &&
        (net->dev_addr[0] & 0x02) == 0)
    strcpy(net->name, "eth%d");
```

`cdc_ether` sets `FLAG_ETHER`, so the deciding test is bit `0x02` of the first
MAC byte. `g_ether` defaults to random **locally administered** MACs, which
sets that bit and forces `usb0` — and nothing on the chumby can use a `usb*`
interface: `add.sh` selects with `awk '/^eth/`, and `network_interface`, which
`network_status.sh` and the panel's readiness check both go through, only ever
considers `eth*` or `rausb*`. `init_pi.sh` pins MACs with the bit clear, and
the chumby then calls the link `eth<n>`.

**2. mountmon derives the module name from the moddef FILENAME.** `add.sh`
greps `vendor:product` against `/etc/mountmon/usbnet-*.moddef`, then loads
`/drivers/<suffix>` for each file that matched. Stock ships only
`usbnet-asix.moddef` and `usbnet-pegasus.moddef`, so `cdc_ether.ko` — present
in `/drivers` all along — is reachable only through a file named
`usbnet-cdc_ether.moddef`. That is what `userhook0` creates. Once loaded,
`cdc_ether` binds by CDC-ECM *class*, so the ID list only has to get the
device recognised.

## What `init_pi.sh` does, and what it deliberately does not

Two changes:

- `/etc/modprobe.d/usb-gadget-mac.conf` — `dev_addr` (Pi end) and `host_addr`
  (chumby end). `host_addr` **must match `GADGET_MAC` in `userhook1`**.
- `/etc/nftables.conf` + `/etc/sysctl.d/99-chumby-forward.conf` +
  `nftables.service` enabled — the masquerade for the gadget subnet.

Everything else on a stock gadget-mode image already works and is left alone:
NetworkManager's `USB Gadget (shared)` profile assigns the address and runs
dnsmasq for DHCP and DNS. Its **only** defect is that it does not install its
nftables NAT at boot — verified on a clean flash: address present,
`nft list tables` empty. Re-activating the profile by hand creates
`table ip nm-shared-usb0`, which is no use unattended.

That is a long-standing bug, not local misconfiguration, and not
Pi-specific — same failure reported across distros for years:

| Report | When | Platform |
|---|---|---|
| [NM #1016](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1016) | 2022 | Ubuntu 22.04, hotspot at boot, fixed by re-activating |
| [NM #1390](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1390) | 2023 | "1 out of 10 times … fails to set up the routing" |
| [NM #1827](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1827) | 2025 | Ubuntu, NM 1.46 |
| [NM #1935](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1935) | 2026 | Mobian, NM 1.56 |
| [Pi forums t=366434](https://forums.raspberrypi.com/viewtopic.php?t=366434) | 2024 | Pi OS, shared `usb0` |
| [trixie-feedback #62](https://github.com/raspberrypi/trixie-feedback/issues/62) | 2026 | Pi OS Trixie |

All unassigned, closed roughly a year after filing, and still present in
NM 1.52.1 here.

**Do not reach for `systemd-networkd`.** It can do address, DHCP and NAT in
one file, and it worked — but on this image NetworkManager owns *both* `wlan0`
and `usb0` (networkd ships inactive and disabled), so enabling it puts a
second manager on the link that keeps the Pi reachable. Doing that once cost
the box: it dropped off wifi and needed a reflash. Nothing here should start,
stop, enable or disable a network manager.

## The chumby side

Root is **cramfs, read-only**; only `/psp` (jffs2) survives a write. `rcS`
runs three hooks and the differences matter:

| Hook | How rcS runs it | Position |
|---|---|---|
| `userhook0` | **sourced** — a stray `exit` kills the boot | before mountmon (rcS:69) and `start_network` (~199) |
| `userhook1` | executed | after `start_network` |
| `userhook2` | executed | late (pre-existing: control-panel cache) |

**Never create `/psp/rfs1/rcS`.** `rcS` runs it and then `exit`s, replacing
the entire remaining boot.

`userhook0` shadows `/etc/mountmon` from tmpfs with the moddef, then waits for
a NIC — but only when none is on the bus yet. A dongle is enumerated before
this point, so a dongle-only boot waits zero seconds; the Pi's gadget does not
answer until its own kernel has loaded `g_ether`, roughly two minutes in, and
mountmon gets exactly one attempt at it. The moddef ids *are* the wait
condition.

`userhook1` runs `start_network`, finds the gadget **by MAC** — the name
varies, `eth1` beside a dongle and `eth0` alone — brings it up, and if nothing
else holds a default route asks for a lease and waits for it. Then
`sync_time.sh -b`, because `rcS` syncs the clock only when the internal wifi
has carrier (`ifconfig rausb0 | grep RUNNING`), which never passes here.

Three vendor behaviours make that necessary: `add.sh` skips the bring-up
whenever another `eth*` is already RUNNING, its `udhcpc -R -n` exits the
instant no server answers, and bringing the interface up is also the ECM
altsetting trigger — the gadget asserts carrier only once the host selects the
data altsetting.

Nothing else is persisted on the chumby. The USB id lives inside
`userhook0`'s moddef heredoc, so `init_chumby.sh` only has to install the two
hooks.

### Getting files onto a chumby with no network

Put a file named `userhook0` in the root of a FAT-formatted USB stick — `rcS`
sources it at boot, and FAT mounts as mode 0755 so the executable test
passes. Have it write the real hooks into `/psp/rfs1/`, boot once with the
stick, then remove it.

## Verify

On the Pi:

```
cat /sys/module/g_ether/parameters/host_addr   # b8:27:eb:5c:f7:9e
ip -br addr show usb0                          # 10.12.194.1/28
sudo nft list table ip chumby_nat              # the masquerade rule
systemctl is-enabled nftables.service          # enabled
```

On the chumby:

```
lsmod | grep cdc_ether        # loaded, via the moddef
ifconfig -a                   # the gadget is eth<n>, not usb0
route -n | grep ^0            # default via 10.12.194.1
ping -c2 8.8.8.8
date                          # clock set by sync_time.sh
```

If the interface is still `usb0`, the MAC pinning did not take.

## Adapting to a different pair

- **MACs**: any pair with bit `0x02` clear. `host_addr` in `init_pi.sh` and
  `GADGET_MAC` in `userhook1` must match.
- **Subnet**: `10.12.194.0/28` comes from the stock NM profile; if you change
  it there, change `LINK_NET` in `init_pi.sh` to match.
- **Gadget id**: verify rather than trusting `2e8a:0013` — a different Pi OS
  release or a legacy `g_ether` build presents different ids. Read it from
  `/sys/bus/usb/devices/*/idVendor` on the chumby.

### Getting a shell on the chumby

There is deliberately no sshd here: the hooks used to start it
(`pidof sshd || /sbin/sshd`) while this was being debugged, and that is gone.
The chumby ships `/sbin/sshd` with host keys and `sshd_config` in
`/usr/local/etc`, but nothing in `rcS` starts it. To get remote access back,
add that one line to `userhook1` — over the network if it is still up, or via
the USB-stick route above if it is not.

## Known limitations

- **A Pi reboot orphans the chumby** until the chumby reboots too: the gadget
  re-enumerates, the chumby's netdev is destroyed and recreated, and its DHCP
  client dies with it. Seen as `RX: 0 packets` on the Pi's `usb0` with carrier
  up. `claude/issues.md` #7.
- **A blackholed route wedges the control panel.** `network_status.sh` fetches
  `http://www.chumby.com/crossdomain…` with no timeout, so a gateway that
  drops traffic makes the panel hang forever and look like a failed boot —
  worse than no route at all, which fails fast. This is what a missing NAT
  looked like. `claude/issues.md` #8.

## Note

The house wifi PSK was exposed during this work and needs rotating; until then
the wifi side is not a trust boundary.
