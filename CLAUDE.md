# Project: Chumby control panel on Raspberry Pi via modified Ruffle

The authoritative plan is in `chumby-ruffle-plan.md`. Read it fully before
doing anything. Follow it step by step, in order.

Non-negotiable rules (duplicated from the plan because they matter most):
- STOP at every CHECKPOINT in the plan. Summarize findings, ask the user,
  and wait for an answer. Never proceed past a checkpoint on your own.
- Every step must end with the engineering record under `claude-docs/`
  updated (`docs/` is end-user documentation). There are exactly three
  documents per repo — `requirements.md`, `design.md`, `development.md` —
  and findings are folded into whichever fits. Do not start a fourth
  file, and do not keep per-session or per-milestone records: what
  survives a milestone is the decision and its reason.
- The docs are split like the code. Anything about the Rust player —
  what the panel demands of it, how the host boundary works, how to
  build/verify/rebase it — belongs in the `ruffle/` submodule's
  `claude-docs/`. Anything about the appliance — packaging, kiosk,
  fixtures, hardware, CI — belongs in this repo's. chumby-pi knows only
  how to build, launch and feed the player; nothing about its internals.
  If a finding fits neither, it is obsolete — say so and ask the user.
- Never modify controlpanel.swf or any extracted SWF.
- /home/jan/chumby_backup is read-only ground truth. Never write there.
- Work on ONE step at a time. Do not look ahead or start the next step's
  work while the current artifact is unfinished.
- One feature branch per working session, squashed on merge — in this
  repo and in the `ruffle/` submodule. Commit after each completed step
  with the step number in the message. Bump the submodule gitlink in the
  same change that needs it. Pushing is the user's call.
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
