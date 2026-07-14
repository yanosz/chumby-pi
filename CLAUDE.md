# Project: Maintenance of Chumby control panel on Raspberry Pi via modified Ruffle

Your task is to support maintenance (e.g. bug-fixes, minor features) of this project.

The project aims at creating a patchset for ruffle to mimic a chumby.
Ruffle is a rust flash emulator. Chumby is smart radio clock (year 2006 - 2012), 
a patchset is needed, because the old flash movie steers the software.

This repository contains all infrastructure, whereas ruffle contains the patchset.

Both repositories contain a folder called claude-docs. These are internal documents that were create when 
the project was active and had not reached maintenance mode, yet. This notes can be helpful for understanding design 
decisions but are not changed any longer.

When being tasked to fix a bug or do a feature, create a plan with steps / checkpoints first.

Non-negotiable rules:
- Work in branch "dev" - both repositories. Default branches are main (chumby-pi) and chumby (ruffle).
  Rebase dev branches to main / chumby when tasked. Work in main / chumby only when requested explicitly. 
- STOP at every CHECKPOINT. Summarize findings, ask the user, and wait for
  an answer. Never proceed past a checkpoint on your own. When scope is
  ambiguous, ask — one clarifying question beats an exploratory detour, and
  a question from the user is a question, not a licence to start coding.
- Every step must end with the engineering record under `claude/`
  updated (`docs/` is end-user documentation). 
- The docs are split like the code. Anything about the Rust player —
  what the panel demands of it, the host boundary, the fixtures, the
  decompiled SWF, how to build/run/verify/rebase it — belongs in the
  `ruffle/` submodule, which is self-contained for player work and has
  its own `CLAUDE.md`. Anything about the appliance — packaging, kiosk,
  hardware, the device record, CI — belongs in this repo's `claude-docs/`.
  If a finding fits neither, it is obsolete — say so and ask the user.
- **Player work happens in `ruffle/`, not here.** If the task is about
  the panel, the natives, the fixtures or the UI policy, work from that
  repo and read its `CLAUDE.md` first.
- Never modify controlpanel.swf or any extracted SWF.
- /home/jan/chumby_backup is read-only ground truth. Never write there.
- Work on ONE step at a time. Do not look ahead or start the next step's
  work while the current step is unfinished.
- - Keep code comments brief. Code should speak for itself; comment only
  what cannot be read from the code — the why, a non-obvious constraint, a
  reference. Do not narrate what the code plainly does.
