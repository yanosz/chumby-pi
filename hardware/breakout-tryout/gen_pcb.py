#!/usr/bin/env python3
"""Build chumby-breakout.kicad_pcb from netlist.xml with pcbnew.

Placement: chumbilical + 5V-out USB on the top edge, everything else on
the bottom edge, all pads facing one horizontal routing channel.
Routing: per-net channel y on B.Cu, vertical escapes on F.Cu, vias at
the junctions. GND is unrouted copper: pours on both layers.
"""
import xml.etree.ElementTree as ET
import pcbnew

FPLIB = "/usr/share/kicad/footprints"
MM = pcbnew.FromMM
def P(x, y): return pcbnew.VECTOR2I(MM(x), MM(y))

BOARD_FILE = "chumby-breakout.kicad_pcb"
LEFT, RIGHT, TOP, BOT = 95, 195, 80, 135
CH_Y0, CH_PITCH = 100.0, 1.2          # channel strip
W_SIG, W_PWR = 0.3, 0.4               # track widths (mm)
VIA_D, VIA_DRILL = 0.7, 0.35

# ref -> (pretty, fp name, pad-field center (x, y), rotation deg, top?)
PLACE = {
    "P1": ("Connector_IDC.pretty", "IDC-Header_2x13_P2.54mm_Vertical", (125.08, 87.5), 90, True),
    "J3": ("Connector_USB.pretty", "USB_A_Kycon_KUSBX-AS1N-B_Horizontal", (176, 88.5), 180, True),
    "J2": ("Connector_PinHeader_2.54mm.pretty", "PinHeader_2x05_P2.54mm_Vertical", (109.84, 128), 90, False),
    "J7": ("Connector_JST.pretty", "JST_PH_S4B-PH-K_1x04_P2.00mm_Horizontal", (130, 128.5), 180, False),
    "J6": ("TerminalBlock.pretty", "TerminalBlock_bornier-4_P5.08mm", (146.5, 128), 0, False),
    "J4": ("Connector_USB.pretty", "USB_A_Molex_48037-2200_Horizontal", (169, 128), 270, False),
    "J5": ("Connector_USB.pretty", "USB_A_Molex_48037-2200_Horizontal", (185, 128), 270, False),
}
HOLES = [(99.5, 84.5), (190.5, 84.5), (99.5, 130.5)]

board = pcbnew.NewBoard(BOARD_FILE)

# ---- nets ---------------------------------------------------------------
root = ET.parse("netlist.xml").getroot()
nets, nodes = {}, {}
for n in root.iter("net"):
    name = n.get("name")
    ni = pcbnew.NETINFO_ITEM(board, name)
    board.Add(ni)
    nets[name] = ni
    nodes[name] = [(x.get("ref"), x.get("pin")) for x in n.iter("node")]

# ---- footprints ---------------------------------------------------------
fps = {}
for ref, (lib, name, (cx, cy), rot, top) in PLACE.items():
    fp = pcbnew.FootprintLoad(f"{FPLIB}/{lib}", name)
    fp.SetReference(ref)
    fp.SetPosition(P(cx, cy))
    fp.SetOrientationDegrees(rot)
    board.Add(fp)
    # shift so the PAD FIELD (not the anchor) is centered on the target
    ps = [p.GetPosition() for p in fp.Pads() if p.GetNumber()]
    mx = (min(p.x for p in ps) + max(p.x for p in ps)) // 2
    my = (min(p.y for p in ps) + max(p.y for p in ps)) // 2
    fp.Move(pcbnew.VECTOR2I(MM(cx) - mx, MM(cy) - my))
    fps[ref] = fp

for name, lst in nodes.items():
    for ref, pin in lst:
        for pad in fps[ref].Pads():
            if pad.GetNumber() == pin:
                pad.SetNet(nets[name])

for i, (hx, hy) in enumerate(HOLES, 1):
    fp = pcbnew.FootprintLoad(f"{FPLIB}/MountingHole.pretty", "MountingHole_3.2mm_M3_Pad_Via")
    fp.SetReference(f"H{i}")
    fp.SetPosition(P(hx, hy))
    for pad in fp.Pads():
        pad.SetNet(nets["GND"])
    board.Add(fp)

# ---- routing ------------------------------------------------------------
def pad_of(ref, pin):
    cands = [p for p in fps[ref].Pads() if p.GetNumber() == pin]
    return min(cands, key=lambda p: abs(p.GetPosition().y - MM(110)))

def add_track(net, a, b, w, layer):
    t = pcbnew.PCB_TRACK(board)
    t.SetStart(a); t.SetEnd(b)
    t.SetWidth(MM(w)); t.SetLayer(layer); t.SetNet(net)
    board.Add(t)

def add_via(net, pos):
    v = pcbnew.PCB_VIA(board)
    v.SetPosition(pos)
    v.SetDrill(MM(VIA_DRILL))
    try:
        v.SetWidth(pcbnew.PADSTACK.ALL_LAYERS, MM(VIA_D))
    except TypeError:
        v.SetWidth(MM(VIA_D))
    v.SetViaType(pcbnew.VIATYPE_THROUGH)
    v.SetLayerPair(pcbnew.F_Cu, pcbnew.B_Cu)
    v.SetNet(net)
    board.Add(v)

def lane_x(ref, pad):
    """Escape-lane x: far row (relative to channel) jogs +1.27 between
    the near row's pads; single-row and USB footprints go straight."""
    ys = sorted({p.GetPosition().y for p in fps[ref].Pads() if p.GetNumber()})
    x, y = pad.GetPosition().x, pad.GetPosition().y
    if len(ys) == 2 and abs(ys[1] - ys[0]) < MM(3):
        top_side = PLACE[ref][4]
        far = ys[0] if top_side else ys[1]
        if y == far:
            return x + MM(1.27)
    return x

def route(net_name, a, b, ch_y, w):
    net = nets[net_name]
    (ra, pa), (rb, pb) = a, b
    for ref, pin in (a, b):
        pad = pad_of(ref, pin)
        lx = lane_x(ref, pad)
        pp = pad.GetPosition()
        if lx != pp.x:  # dogleg toward the lane
            add_track(net, pp, pcbnew.VECTOR2I(lx, pp.y + (MM(1.27) if pp.y < MM(110) else -MM(1.27))), w, pcbnew.F_Cu)
            pp = pcbnew.VECTOR2I(lx, pp.y + (MM(1.27) if pp.y < MM(110) else -MM(1.27)))
        v = pcbnew.VECTOR2I(lx, MM(ch_y))
        add_track(net, pp, v, w, pcbnew.F_Cu)
        add_via(net, v)
    xa = lane_x(ra, pad_of(ra, pa)); xb = lane_x(rb, pad_of(rb, pb))
    add_track(net, pcbnew.VECTOR2I(xa, MM(ch_y)), pcbnew.VECTOR2I(xb, MM(ch_y)), w, pcbnew.B_Cu)

spans = []
for name, lst in nodes.items():
    if name == "GND" or name.startswith("unconnected-"):
        continue
    lst = sorted(lst)
    hub, rest = lst[0], lst[1:]
    for other in rest:                 # star from first node
        spans.append((name, hub, other))

# A descending drop (from a top pad) crosses every channel above its own;
# an ascending drop (to a bottom pad) crosses every channel below its own.
# If a descending and an ascending drop are closer than MIN_LANE_GAP in x,
# the descending net's channel must lie ABOVE the ascending net's.
MIN_LANE_GAP = MM(0.70)   # via radius + track half-width + clearance
def drops(span):
    desc, asc = [], []
    for ref, pin in span[1:]:
        pad = pad_of(ref, pin)
        (desc if pad.GetPosition().y < MM(110) else asc).append(lane_x(ref, pad))
    return desc, asc

after = {i: set() for i in range(len(spans))}  # i's channel above j's
for i, si in enumerate(spans):
    di, _ = drops(si)
    for j, sj in enumerate(spans):
        if i == j:
            continue
        _, aj = drops(sj)
        if any(abs(d - a) < MIN_LANE_GAP for d in di for a in aj):
            after[j].add(i)            # ch_i < ch_j
order = []
while len(order) < len(spans):
    ready = [i for i in after if not after[i] and i not in order]
    if not ready:
        break
    for n in ready:
        order.append(n)
        for j in after:
            after[j].discard(n)
if len(order) != len(spans):
    raise SystemExit("channel constraint cycle - adjust placement")

for slot, idx in enumerate(order):
    name, a, b = spans[idx]
    w = W_PWR if name in ("+5V", "+3V3") else W_SIG
    route(name, a, b, CH_Y0 + slot * CH_PITCH, w)

# ---- edge, zones, silk --------------------------------------------------
edge = pcbnew.PCB_SHAPE(board)
edge.SetShape(pcbnew.SHAPE_T_RECT)
edge.SetStart(P(LEFT, TOP)); edge.SetEnd(P(RIGHT, BOT))
edge.SetLayer(pcbnew.Edge_Cuts)
edge.SetWidth(MM(0.1))
board.Add(edge)

for layer in (pcbnew.F_Cu, pcbnew.B_Cu):
    z = pcbnew.ZONE(board)
    z.SetLayer(layer)
    z.SetNet(nets["GND"])
    z.Outline().NewOutline()
    for x, y in ((LEFT, TOP), (RIGHT, TOP), (RIGHT, BOT), (LEFT, BOT)):
        z.Outline().Append(MM(x), MM(y))
    z.SetMinThickness(MM(0.25))
    board.Add(z)
pcbnew.ZONE_FILLER(board).Fill(board.Zones())

def silk(txt, x, y, size=1.2, layer=pcbnew.F_SilkS):
    t = pcbnew.PCB_TEXT(board)
    t.SetText(txt)
    t.SetPosition(P(x, y))
    t.SetTextSize(pcbnew.VECTOR2I(MM(size), MM(size)))
    t.SetTextThickness(MM(size * 0.15))
    t.SetLayer(layer)
    board.Add(t)

silk("CHUMBY DAUGHTERCARD BREAKOUT (TRYOUT rev 0.0)", 145, 96, 1.5)
silk("5V IN ONLY - THE ORIGINAL 12V WART FRIES THE PI", 145, 98, 1.2)
silk("chumbilical", 120, 82)
silk("5V to Pi", 176, 82)
silk("Pi 17-26", 110, 124)
silk("SPK", 130, 124)
silk("HP L/R/G/DET", 150, 124)
silk("hub A", 167, 124)
silk("hub B", 182, 124)

pcbnew.SaveBoard(BOARD_FILE, board)
print(f"saved {BOARD_FILE}: {len(spans)} routed spans, "
      f"{len(board.GetTracks())} track items")
