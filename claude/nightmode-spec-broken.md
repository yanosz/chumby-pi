# The night-mode spec is broken — parked 2026-11-05

`claude/nightmode-spec.md` does not model the problem properly. **Do not
implement from it.** Jan stopped the session on it (2026-11-05, evening):
too much friction, and the modelling questions below need answering first,
probably with proper UML rather than an ad-hoc state diagram.

## What is wrong with it (Jan's words, 2026-11-05)

1. **"Unload" is a technical event, not a domain event.** The spec leans on
   Lys being loaded or unloaded, which is a fact about the player, not about
   the appliance's behaviour. The domain concept behind it has to be named
   and defined before it can appear in a machine — and the spec's prose
   workaround ("what does not move the state") only hides the gap.
2. **"The panel enters night mode" does not say which panel, or which night
   mode.** The label collapses several different things, and the reader
   cannot tell them apart.
3. **There are probably more states than three.** Something like "Lys
   active" may well be a state of its own — the current model carries it as
   an unstated condition, which is why edges kept needing guards, and why
   the restart out of Lys night behaves unlike every other edge.

## Where things stand

- **Nothing is implemented, deployed or pushed.** Step 1's half-finished
  code was discarded at Jan's request (2026-11-05): the `rust-remote` repo
  is back to `b627f0e` on `main` with a clean working tree, and the branch
  that held the work-in-progress is deleted. It is recoverable from the
  reflog as `a35e507` for about 90 days, but it implements the rejected
  model and should be rewritten rather than revived.
- On chumby-pi `dev`: the research under issue 32, the fix plan, and the
  spec with its revisions — commits `48e0090`, `1be1d6d`, `3ec5742`,
  `a698e7d`, `3b93ca1`, `352e121`, `08df86b`, `e423003`. Nothing pushed.
- The record's findings are sound and independent of the modelling: the
  night lost at 04:22 on 2026-11-05 was Lys's, not the panel's; `2` is
  invisible to the panel because both of its reads are `== "1"` string
  compares; a `1` cannot be held during a widget night (the idle frame
  re-reads it and flip-flops at 2 s); Lys already announces its night to
  the remote through `screen dim` and `screen bright`; and the supervisor
  can push a panel-start instead of anyone polling. Those survive whatever
  the model turns out to be.

## For the next session

Start from the domain, not from the file: what are the appliance's night
states as a user would name them, which events are domain events, and only
then which byte in `/tmp/nightmode` encodes what. The automaton in
`nightmode-spec.md` is at best a sketch of the file's behaviour, and should
be rewritten rather than patched again.
