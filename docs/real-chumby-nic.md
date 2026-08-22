# Giving a real chumby a network through a Raspberry Pi

A chumby whose own wifi has died can still get online: a Raspberry Pi with
USB-OTG becomes its network card. The Pi joins your wifi, presents a USB
ethernet gadget to the chumby, and routes between the two. The chumby sees an
ordinary USB ethernet adapter.

This is not the same setup as [setup.md](setup.md), which turns a Pi *into* a
chumby. Here the chumby stays the chumby.

Reference hardware: Raspberry Pi Zero W (any Pi with a USB-OTG port works),
Raspberry Pi OS Lite trixie, a real chumby with a free USB host port, and a
USB-A-to-micro-B cable. Verified against a chumby running 2.6.16-csb.

Two facts drive nearly every step, and neither is guessable:

- **The chumby names the interface from its MAC.** `usbnet` calls the link
  `usb0` unless the MAC's locally-administered bit is clear, and *nothing* on
  the chumby can use a `usb*` interface — its `network_interface` helper, and
  so the whole status/readiness path, only ever looks at `eth*` or `rausb*`.
  Pinning the gadget's MACs is therefore mandatory, not cosmetic.
- **The chumby's root filesystem is read-only** (cramfs). Only `/psp` (jffs2)
  survives a write, so every chumby-side change goes there.

## Part 1 — the Pi

Assumes wifi already works on the Pi (`nmcli device status` shows `wlan0`
connected). Configure that yourself; don't put credentials in files you
share.

### 1. Enable the USB gadget

Pi OS ships `g_ether` autoloading via `/etc/modules-load.d/usb-gadget.conf`.
The OTG port must be in peripheral (or `otg`) mode, which needs a line in
`/boot/firmware/config.txt`:

```
dtoverlay=dwc2,dr_mode=peripheral
```

Reboot and confirm a gadget exists:

```
ls /sys/class/udc/          # e.g. 20980000.usb
ip -br link show usb0
```

Note the cost: that port becomes the gadget, so the Pi can no longer host a
keyboard, and its HDMI console is read-only from then on. Keep ssh working.

### 2. Pin the gadget MACs — the step everything else depends on

```
sudo tee /etc/modprobe.d/usb-gadget-mac.conf >/dev/null <<'EOF'
# usbnet on the host names the link usb%d unless FLAG_ETHER is set AND
# (dev_addr[0] & 0x02) == 0. g_ether otherwise picks random locally
# administered MACs, which forces usb0 on the chumby - an interface its
# scripts can never use. Clearing that bit makes the chumby name it eth<n>.
options g_ether dev_addr=b8:27:eb:5c:f7:9d host_addr=b8:27:eb:5c:f7:9e
EOF
sudo modprobe -r g_ether && sudo modprobe g_ether
```

`dev_addr` is the Pi's end, `host_addr` the chumby's. Any address with the
`0x02` bit clear works; `b8:27:eb` is the Raspberry Pi OUI, and picking
values adjacent to the Pi's own `wlan0` keeps them traceable. **Write down
`host_addr`** — Part 2 needs it.

### 3. Give the link to systemd-networkd, not NetworkManager

NetworkManager's `ipv4.method=shared` is unreliable here: it reports the
connection activated while silently skipping its nftables stage at boot, so
there is no NAT until you re-activate the profile by hand. That is a
long-standing cross-distro bug, still present in NM 1.52.1 — see
`claude/pi-as-chumby-nic.md` for the reports. Use networkd instead, which
applies a static file at link-up:

```
sudo tee /etc/systemd/network/10-usb0.network >/dev/null <<'EOF'
[Match]
Name=usb0

[Link]
# Never let boot block on the chumby being plugged in.
RequiredForOnline=no

[Network]
Address=10.12.194.1/28
IPMasquerade=ipv4
IPv4Forwarding=yes
DHCPServer=yes

[DHCPServer]
PoolOffset=2
PoolSize=12
EmitDNS=yes
DNS=8.8.8.8
EOF

sudo tee /etc/NetworkManager/conf.d/99-unmanaged-usb0.conf >/dev/null <<'EOF'
[keyfile]
unmanaged-devices=interface-name:usb0
EOF

# The stock ICS switcher only makes sense while NM owns usb0.
sudo systemctl disable --now rpi-usb-gadget-ics.service
sudo nmcli general reload conf
sudo systemctl enable --now systemd-networkd
# Enabling networkd pulls in wait-online, which then fails: the only managed
# link is deliberately RequiredForOnline=no. NM-wait-online covers wlan0.
sudo systemctl disable --now systemd-networkd-wait-online
```

`IPMasquerade=ipv4` writes the NAT itself and `DHCPServer=yes` replaces
dnsmasq, so networkd covers address, DHCP and NAT in one file.

### 4. Verify the Pi

```
nmcli -t -f DEVICE,STATE device status | grep usb0   # -> usb0:unmanaged
ip -br addr show usb0                                # -> 10.12.194.1/28
sudo nft list tables                                 # -> table ip io.systemd.nat
networkctl status usb0                               # -> routable (configured)
```

Reboot and check `nft list tables` again — that the NAT is present with no
manual step is the whole point of using networkd.

## Part 2 — the chumby

Everything here lives in `/psp/rfs1/`. `rcS` runs three hooks and the
differences matter:

| Hook | How rcS runs it | Position |
|---|---|---|
| `userhook0` | **sourced** — a stray `exit` kills the boot | before mountmon and before the network starts |
| `userhook1` | executed | after the network has started |
| `userhook2` | executed | late |

**Never create `/psp/rfs1/rcS`.** `rcS` runs it and then `exit`s, replacing
the entire remaining boot.

### 5. Getting files onto a chumby with no network

If the chumby has no working network yet, use a FAT-formatted USB stick: put
a file named `userhook0` in its root and `rcS` will source it at boot (FAT
mounts as mode 0755, so the executable test passes). Have that stick script
write the real hooks into `/psp/rfs1/` — one boot with the stick, then remove
it. Otherwise scp them straight into `/psp/rfs1/`.

### 6. `userhook0` — teach mountmon the gadget, and wait for it

Two jobs. First, `add.sh` derives the driver name from the moddef
*filename*, so `cdc_ether.ko` — already present in `/drivers` — is only
reachable through a file called `usbnet-cdc_ether.moddef`. The stock
`/etc/mountmon` is read-only, so shadow it from tmpfs. Second, wait for a NIC
only when none is on the bus yet: a dongle is enumerated before this point,
while the Pi's gadget does not answer until its own kernel has loaded
`g_ether` roughly two minutes in — and mountmon gets exactly one attempt at
it.

```sh
#!/bin/sh
# Sourced by rcS - must never call exit.

if [ ! -f /etc/mountmon/usbnet-cdc_ether.moddef ]; then
	mkdir -p /tmp/mountmon
	cp /etc/mountmon/* /tmp/mountmon/
	cat > /tmp/mountmon/usbnet-cdc_ether.moddef <<"MODDEF"
2e8a:0013	# Raspberry Pi USB Gadget, CDC-ECM
0525:a4a1	# legacy g_ether CDC-ECM
0525:a4a2	# legacy g_ether RNDIS / subset
049f:505a	# legacy g_ether CDC subset
MODDEF
	mount -o bind /tmp/mountmon /etc/mountmon
fi

nic_on_bus() {
	for f in /sys/bus/usb/devices/*/idVendor; do
		[ -f "$f" ] || continue
		v=$(cat "$f" 2>/dev/null)
		p=$(cat "$(dirname "$f")/idProduct" 2>/dev/null)
		[ -n "$v" ] && [ -n "$p" ] || continue
		grep -qi "^$v:$p" /etc/mountmon/usbnet-*.moddef 2>/dev/null && return 0
	done
	return 1
}

i=0
while [ $i -lt 240 ]; do
	nic_on_bus && break
	sleep 2
	i=$((i + 2))
done
```

`2e8a:0013` is what Raspberry Pi OS presents; confirm yours with
`cat /sys/bus/usb/devices/*/idVendor` on the chumby, or macOS System
Information → USB. The other ids are legacy `g_ether` defaults, harmless to
keep. Because the id list *is* the wait condition, a dongle-only boot never
waits — its id is in the stock moddefs.

### 7. `userhook1` — sshd, the gadget's address, and the clock

Three vendor behaviours make this necessary. `add.sh` selects with
`awk '/^eth/'` and skips the bring-up whenever another `eth*` is already
RUNNING; its `udhcpc -R -n` exits the instant no server answers; and `rcS`
syncs time only when the *internal wifi* has carrier
(`ifconfig rausb0 | grep RUNNING`), which never passes here.

Set `GADGET_MAC` to the `host_addr` from step 2.

```sh
#!/bin/sh
GADGET_MAC=B8:27:EB:5C:F7:9E

gadget_if() {
	for n in $(ifconfig -a 2>/dev/null | sed -n "s/^\([^ ]*\) .*Link encap.*/\1/p"); do
		if ifconfig "$n" 2>/dev/null | grep -qi "HWaddr $GADGET_MAC"; then
			echo "$n"; return 0
		fi
	done
	return 1
}
has_addr() { ifconfig "$1" 2>/dev/null | grep -q "inet addr"; }
have_default() { route -n 2>/dev/null | grep -q "^0\.0\.0\.0"; }

# Host keys and sshd_config ship in /usr/local/etc; nothing in rcS starts it.
pidof sshd >/dev/null 2>&1 || /sbin/sshd

/usr/chumby/scripts/start_network

IF=$(gadget_if)
if [ -n "$IF" ]; then
	# Also the altsetting trigger: an ECM gadget asserts carrier only once
	# the host selects the data altsetting.
	ifconfig "$IF" up
	if ! has_addr "$IF" && ! have_default; then
		udhcpc -R -b -p /var/run/udhcpc."$IF".pid -i "$IF" &
		i=0
		while [ $i -lt 90 ]; do
			has_addr "$IF" && break
			sleep 2
			i=$((i + 2))
		done
	fi
fi

/usr/chumby/scripts/sync_time.sh -b &
```

Matching on the MAC rather than the name is deliberate: the gadget lands on
`eth1` while a dongle occupies `eth0`, and on `eth0` when it is alone. The
`have_default` guard means a temporarily attached dongle keeps its route
instead of having it stolen.

Make both hooks executable (`chmod +x /psp/rfs1/userhook0 /psp/rfs1/userhook1`)
and syntax-check them on the device (`sh -n /psp/rfs1/userhook0`).

### 8. Verify the chumby

```
lsmod | grep cdc_ether        # loaded, via the moddef
ifconfig -a                   # the gadget appears as eth<n>, not usb0
route -n | grep ^0            # default route via 10.12.194.1
cat /etc/resolv.conf          # nameserver 8.8.8.8
ping -c 2 8.8.8.8
wget -T 10 -O /dev/null http://www.chumby.com/crossdomain.xml
date                          # clock set by sync_time.sh
```

If the interface is still called `usb0`, the MAC pinning in step 2 did not
take — check `cat /sys/module/g_ether/parameters/host_addr` on the Pi.

## Adapting to a different pair

- **MACs**: any pair with the `0x02` bit clear. `host_addr` and the chumby's
  `GADGET_MAC` must match.
- **Subnet/DNS**: `10.12.194.1/28` and `8.8.8.8` are arbitrary; change both
  in `10-usb0.network`. The `/28` gives twelve leases, plenty for one chumby.
- **Gadget id**: verify it rather than trusting `2e8a:0013`; a different Pi
  OS release or a legacy `g_ether` build presents different ids.

## Known limitations

- **A Pi reboot orphans the chumby** until the chumby also reboots: the
  gadget re-enumerates, the chumby's netdev is destroyed and recreated, and
  its DHCP client dies with it. Reboot the chumby after the Pi.
- **A blackholed route wedges the control panel.**
  `network_status.sh` fetches `http://www.chumby.com/crossdomain…` with no
  timeout, so a route whose gateway drops traffic makes the panel hang
  forever and look like a failed boot. It is worse than having no route at
  all, which fails fast.
