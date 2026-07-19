# chumby-breakout — generated KiCad tryout

A **skills tryout, not the engineering record**: the issue-2 breakout
board (claude/issues.md) generated end-to-end by script, 2026-07-17.
The folder is gitignored. Decisions still open in issue 2 (P50V
beep-out, power switch, HP_NOTIN, cable-end housing) are baked in here
as assumptions — do not fab from this.

**2026-07-19: gen_sch.py's P1 net map is known wrong** — it inherits a
one-row misread of the schematic, since corrected by measurement
(`../chumby-hat/accelerometer.md` §3: reset = sch 9↔11, bend line =
sch 17). Regenerate only after taking the corrected table.

- `gen_sch.py` → `chumby-breakout.kicad_sch` — symbols pulled from the
  stock KiCad 9 libs, nets as global labels at computed pin endpoints.
  **ERC: 0 violations** (`--severity-all`).
- `gen_pcb.py` → `chumby-breakout.kicad_pcb` — placement plus a small
  two-layer channel router (vertical escapes on F.Cu, one horizontal
  channel per net on B.Cu, topologically ordered to avoid crossing
  conflicts), GND pours both sides, plated GND mounting holes, silk.
  **DRC: 0 errors, 0 unconnected**; warnings remaining: silk overlaps
  near J6, and the holes-co-located notice inherent to the
  `_Pad_Via` mounting-hole footprint.
- `chumby-breakout-sch.pdf`, `render-top.png`, `render-bottom.png`,
  `gerbers/` — exported views.

Regenerate: `python3 gen_sch.py && kicad-cli sch export netlist
--format kicadxml -o netlist.xml chumby-breakout.kicad_sch &&
python3 gen_pcb.py`.

Board: 100×55 mm. Top edge: P1 chumbilical (2×13 IDC), J3 USB-A
power-out to the Pi. Bottom edge: J2 (2×5 to Pi pins 17–26), J7
speakers (PH2.0 footprint), J6 headphone screw terminal, J4/J5 USB-A
males to the hub (data+GND only, VBUS open by design).
