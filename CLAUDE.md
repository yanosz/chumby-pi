# Project: Chumby control panel on Raspberry Pi via modified Ruffle

The authoritative plan is in `chumby-ruffle-plan.md`. Read it fully before
doing anything. Follow it step by step, in order.

Non-negotiable rules (duplicated from the plan because they matter most):
- STOP at every CHECKPOINT in the plan. Summarize findings, ask the user,
  and wait for an answer. Never proceed past a checkpoint on your own.
- Every step must end in the named written artifact under claude-docs/
  (the internal engineering record; `docs/` is end-user documentation).
  If you can't produce it, write down why and ask the user.
- Never modify controlpanel.swf or any extracted SWF.
- /home/jan/chumby_backup is read-only ground truth. Never write there.
- Work on ONE step at a time. Do not look ahead or start the next step's
  work while the current artifact is unfinished.
- Commit to git after each completed step with the step number in the message.
- Keep code comments brief. Code should speak for itself; comment only
  what cannot be read from the code — the why, a non-obvious constraint, a
  reference. Do not narrate what the code plainly does.
- Every operation performed on the Raspberry Pi (packages installed,
  config.txt/overlay changes, systemd units, sysfs writes, build steps,
  anything typed over SSH that changes device state) must be documented
  as it happens — command, why, and result — in the relevant
  `claude-docs/reference/*.md` file. The goal is that a "make your
  Raspberry Pi into a Chumby" howto can be assembled from these docs after the fact
  without re-deriving anything from memory or shell history.
