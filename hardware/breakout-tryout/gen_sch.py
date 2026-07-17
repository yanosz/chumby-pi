#!/usr/bin/env python3
"""Generate chumby-breakout.kicad_sch from the stock KiCad 9 symbol libs.

Connectivity comes from global labels placed at computed pin endpoints,
so the schematic is ERC-checkable and netlist-exportable without drawn
wires. Net names follow the chumbilical record in
hardware/chumby-hat/README.md (issue 2 in claude/issues.md).
"""
import re, uuid, math

LIBDIR = "/usr/share/kicad/symbols"
OUT = "chumby-breakout.kicad_sch"
PROJECT = "chumby-breakout"

def u():
    return str(uuid.uuid4())

def extract_symbol(libfile, name):
    txt = open(f"{LIBDIR}/{libfile}").read()
    i = txt.index(f'(symbol "{name}"')
    d = 0
    for j, c in enumerate(txt[i:], i):
        if c == "(":
            d += 1
        elif c == ")":
            d -= 1
        if d == 0:
            break
    return txt[i : j + 1]

def pins_of(sym_sexp):
    """[(number, x, y, angle, length)] in library (y-up) coords."""
    out = []
    for m in re.finditer(
        r'\(pin \w+ \w+\s*\(at ([-\d.]+) ([-\d.]+) (\d+)\)\s*'
        r'\(length ([\d.]+)\).*?\(number "([^"]+)"',
        sym_sexp, re.S):
        x, y, a, l, num = m.groups()
        out.append((num, float(x), float(y), int(a), float(l)))
    return out

# ---- symbols used -------------------------------------------------------
SYMS = {  # lib_id -> (libfile, symbol name)
    "Connector_Generic:Conn_02x13_Odd_Even": ("Connector_Generic.kicad_sym", "Conn_02x13_Odd_Even"),
    "Connector_Generic:Conn_02x05_Odd_Even": ("Connector_Generic.kicad_sym", "Conn_02x05_Odd_Even"),
    "Connector_Generic:Conn_01x04":          ("Connector_Generic.kicad_sym", "Conn_01x04"),
    "Connector:USB_A":                       ("Connector.kicad_sym", "USB_A"),
    "Connector:Screw_Terminal_01x04":        ("Connector.kicad_sym", "Screw_Terminal_01x04"),
    "power:PWR_FLAG":                        ("power.kicad_sym", "PWR_FLAG"),
}
symdefs, sympins = {}, {}
for lib_id, (f, n) in SYMS.items():
    s = extract_symbol(f, n)
    if "(extends" in s.split("\n")[1]:
        raise SystemExit(f"{n} extends another symbol; not handled")
    symdefs[lib_id] = s.replace(f'(symbol "{n}"', f'(symbol "{lib_id}"', 1)
    sympins[lib_id] = pins_of(s)

# ---- instances ----------------------------------------------------------
# ref: (lib_id, x, y, value, footprint, {pin: net or None=NC})
FP = {
    "P1": "Connector_IDC:IDC-Header_2x13_P2.54mm_Vertical",
    "J2": "Connector_PinHeader_2.54mm:PinHeader_2x05_P2.54mm_Vertical",
    "J3": "Connector_USB:USB_A_Kycon_KUSBX-AS1N-B_Horizontal",
    "J4": "Connector_USB:USB_A_Molex_48037-2200_Horizontal",
    "J5": "Connector_USB:USB_A_Molex_48037-2200_Horizontal",
    "J6": "TerminalBlock:TerminalBlock_bornier-4_P5.08mm",
    "J7": "Connector_JST:JST_PH_S4B-PH-K_1x04_P2.00mm_Horizontal",
    "#FLG01": "", "#FLG02": "", "#FLG03": "",
}
G = 1.27  # connection grid; all placements must be multiples
INSTANCES = {
    # chumbilical, nets per hardware/chumby-hat/README.md pin table
    "P1": ("Connector_Generic:Conn_02x13_Odd_Even", 69.85, 99.06, "chumbilical 2x13", {
        "1": None,              # BATTERY - unused
        "2": "+5V",             # RAW_PWR - 5V IN via daughtercard DC jack
        "3": "CSPI1_MISO", "4": "CSPI1_SCLK", "5": "CSPI1_MOSI",
        "6": "CSPI1_SS0", "7": "CHUMBY_RESET_REQ", "8": "CSPI1_SS1",
        "9": "+3V3",            # P33VBKUP (presumed SPI-chip supply)
        "10": None, "11": "USBB_DN", "12": "USBB_DP",
        "13": "USBB2_DN", "14": "USBB2_DP", "15": "CHUMBY_BEND",
        "16": "+5V",            # P50V - both USB VBUS, fed locally
        "17": "HP_NOTIN", "18": "HPLEFT", "19": None, "20": "HPRIGHT",
        "21": "SPKL_VO2", "22": "SPKR_VO2", "23": "SPKL_VO1",
        "24": "SPKR_VO1", "25": "GND", "26": "GND",  # 26 = EMI_GND shield
    }),
    # single cable to Pi header pins 17..26 (J2 pin n = Pi pin 16+n)
    "J2": ("Connector_Generic:Conn_02x05_Odd_Even", 190.5, 59.69, "to Pi 17-26", {
        "1": "+3V3", "2": "CHUMBY_BEND", "3": "CSPI1_MOSI", "4": "GND",
        "5": "CSPI1_MISO", "6": "CHUMBY_RESET_REQ", "7": "CSPI1_SCLK",
        "8": "CSPI1_SS0", "9": "GND", "10": "CSPI1_SS1",
    }),
    # power-only USB-A: 5V arriving at the DC jack leaves here to the Pi
    "J3": ("Connector:USB_A", 129.54, 59.69, "5V out to Pi", {
        "1": "+5V", "2": None, "3": None, "4": "GND", "5": "GND",
    }),
    # USB-A males to the hub: data+GND only, VBUS deliberately open
    "J4": ("Connector:USB_A", 129.54, 105.41, "to hub (no VBUS)", {
        "1": None, "2": "USBB_DN", "3": "USBB_DP", "4": "GND", "5": "GND",
    }),
    "J5": ("Connector:USB_A", 129.54, 144.78, "to hub (no VBUS)", {
        "1": None, "2": "USBB2_DN", "3": "USBB2_DP", "4": "GND", "5": "GND",
    }),
    "J6": ("Connector:Screw_Terminal_01x04", 190.5, 105.41, "headphone", {
        "1": "HPLEFT", "2": "HPRIGHT", "3": "GND", "4": "HP_NOTIN",
    }),
    "J7": ("Connector_Generic:Conn_01x04", 190.5, 144.78, "speakers PH2.0", {
        "1": "SPKL_VO1", "2": "SPKL_VO2", "3": "SPKR_VO1", "4": "SPKR_VO2",
    }),
    "#FLG01": ("power:PWR_FLAG", 39.37, 39.37, "", {"1": "+5V"}),
    "#FLG02": ("power:PWR_FLAG", 59.69, 39.37, "", {"1": "GND"}),
    "#FLG03": ("power:PWR_FLAG", 80.01, 39.37, "", {"1": "+3V3"}),
}

root_uuid = u()
body = []

for ref, (lib_id, X, Y, val, netmap) in INSTANCES.items():
    su = u()
    pin_lines = "\n".join(f'    (pin "{n}" (uuid "{u()}"))' for n, *_ in sympins[lib_id])
    body.append(f'''  (symbol
    (lib_id "{lib_id}")
    (at {X} {Y} 0)
    (unit 1)
    (exclude_from_sim no) (in_bom yes) (on_board yes) (dnp no)
    (uuid "{su}")
    (property "Reference" "{ref}" (at {X} {Y - 22} 0)
      (effects (font (size 1.27 1.27))))
    (property "Value" "{val}" (at {X} {Y + 22} 0)
      (effects (font (size 1.27 1.27))))
    (property "Footprint" "{FP[ref]}" (at {X} {Y} 0)
      (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Datasheet" "~" (at {X} {Y} 0)
      (effects (font (size 1.27 1.27)) (hide yes)))
{pin_lines}
    (instances (project "{PROJECT}"
      (path "/{root_uuid}" (reference "{ref}") (unit 1))))
  )''')
    for num, px, py, ang, _l in sympins[lib_id]:
        gx, gy = X + px, Y - py           # library is y-up, schematic y-down
        net = netmap[num]
        if net is None:
            body.append(f'  (no_connect (at {gx} {gy}) (uuid "{u()}"))')
            continue
        out_lib = (ang + 180) % 360       # pins point INTO the body
        sch_ang = out_lib if out_lib in (0, 180) else (360 - out_lib) % 360
        body.append(f'''  (global_label "{net}" (shape passive)
    (at {gx} {gy} {sch_ang})
    (effects (font (size 1.27 1.27)))
    (uuid "{u()}")
    (property "Intersheetrefs" "${{INTERSHEET_REFS}}" (at {gx} {gy} 0)
      (effects (font (size 1.27 1.27)) (hide yes))))''')

NOTES = """DESIGN NOTES (issue 2, claude/issues.md)\\n\\n\
- P1: chumbilical. Pin numbers PROVISIONAL until the cable is beeped out.\\n\
- 5 V ONLY into the daughtercard DC jack. The original 12 V wall wart WILL FRY the Pi.\\n\
- J4/J5 carry data+GND only; P50V feeds both daughtercard USB jacks from the local 5 V rail.\\n\
- J2 mates Raspberry Pi header pins 17-26 1:1 (3V3, SPI0+CE0/CE1, 2x GND, GPIO24=bend, GPIO25=reset).\\n\
- Bend moves off FR3's GPIO17 -> gpio-key config change.\\n\
- HP_NOTIN placement still undecided; parked on the screw terminal."""
body.append(f'''  (text "{NOTES}"
    (exclude_from_sim no)
    (at 30 170 0)
    (effects (font (size 1.6 1.6)) (justify left bottom))
    (uuid "{u()}"))''')

libsec = "\n".join(symdefs.values())
doc = f'''(kicad_sch
  (version 20250114)
  (generator "eeschema")
  (generator_version "9.0")
  (uuid "{root_uuid}")
  (paper "A4")
  (title_block
    (title "Chumby daughtercard breakout (TRYOUT)")
    (date "2026-07-17")
    (rev "0.0")
    (company "chumby-pi")
    (comment 1 "Generated file - see gen_sch.py; not the engineering record")
  )
  (lib_symbols
{libsec}
  )
{chr(10).join(body)}
  (sheet_instances (path "/" (page "1")))
)
'''
open(OUT, "w").write(doc)
print(f"wrote {OUT}: {len(INSTANCES)} symbols, "
      f"{sum(len(p) for p in sympins.values())} pins")
