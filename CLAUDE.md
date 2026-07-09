# Project: Chumby control panel on Raspberry Pi via modified Ruffle

`ROADMAP.md` is what is left, what is done, and the decisions that still
bind. Read it first — it is short. It was compressed from a much longer
plan and says so: treat it as a map, not as evidence. When it, a doc, and
the code disagree, **the code wins** — grep for the artifact before
believing a claim about it.

Non-negotiable rules:
- STOP at every CHECKPOINT. Summarize findings, ask the user, and wait for
  an answer. Never proceed past a checkpoint on your own. When scope is
  ambiguous, ask — one clarifying question beats an exploratory detour, and
  a question from the user is a question, not a licence to start coding.
- Every step must end with the engineering record under `claude-docs/`
  updated (`docs/` is end-user documentation). There are exactly three
  documents per repo — `requirements.md`, `design.md`, `development.md` —
  and findings are folded into whichever fits. Do not start a fourth
  file, and do not keep per-session or per-milestone records: what
  survives a milestone is the decision and its reason.
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
- Scope creep guard: before implementing support for a panel screen, check
  it against the scope table (`claude-docs/requirements.md` §1 FR5) and ask
  if it is not listed.
- One feature branch per working session — in this repo and in the
  `ruffle/` submodule. **Finishing the session means pushing it and opening
  the pull request** in each repo you touched; do that yourself. Jan reviews
  and squash-merges. Commit after each completed step. Bump the submodule
  gitlink in the same change that needs it, and merge the fork's PR before
  this repo's (squashing replaces the commit the gitlink pins — see
  `claude-docs/development.md` §1).
- Keep code comments brief. Code should speak for itself; comment only
  what cannot be read from the code — the why, a non-obvious constraint, a
  reference. Do not narrate what the code plainly does.
- Every operation performed on the Raspberry Pi (packages installed,
  config.txt/overlay changes, systemd units, sysfs writes, build steps,
  anything typed over SSH that changes device state) must be documented
  as it happens — command, why, and result — in the device record,
  `claude-docs/development.md`. The goal is that a "make your Raspberry
  Pi into a Chumby" howto can be assembled from it after the fact without
  re-deriving anything from memory or shell history.
