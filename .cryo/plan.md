# Plan: many-body-advisor (weekly literature scan)

## Goal

Once a week — every **Monday 09:48 Asia/Shanghai** — wake up, run the
self-contained advisor workflow in this plan, then hibernate. The workflow handles
all the actual work: scanning arXiv, web search, and watched GitHub repos,
deduping against `.knowledge/INDEX.md`, and posting the weekly digest to the
`ManyBodyHarness` Zulip stream via the project's `make zulip-send` bridge.

Project root is `..` (one level up from this chamber). Chamber memory lives in
this directory's `NOTES.md`.

Plan never terminates — the chamber reschedules itself every week. Only the
human stops it (`cryo cancel` or `cryo clean`).

## Tasks

Each session, run these steps in order:

### Generation policy — draft inline

Whenever this plan asks the agent to *generate prose* (the Step 4 outbox
message; any ad-hoc reply text), the agent generates it inline using its own
reasoning. Keep it short — this is orchestration, not a writing task.

Do **not** shell out to `codex exec`: it loads its own skills and runs searches
before producing output, reliably blowing past a 60s timeout for what is a
one-liner. The inline path is faster and deterministic.

Scope: **prose only**. Never let prose drafting touch decisions, scheduling
logic, sanity checks, or tool calls (`make zulip-send`, `cryo-agent *`) — those
are the chamber agent's job regardless.

Keep all prompt files, drafts, and scratch files inside this project. Preferred
locations are `.cryo/tmp/` for chamber scratch and `../.zulip/.drafts/` for
advisor drafts. Do not write to `/tmp`; unattended `opencode` sessions may
auto-reject external-directory access and exit before hibernating.

### Step 0 — Pre-flight iCloud integrity check

Before reading any project file, verify that critical files
(`../AGENTS.md`, `Makefile`) are not iCloud-evicted.
macOS may evict files to `dataless` state when local disk is pressured,
causing all read tools to fail with "Resource deadlock avoided" or
"readAlloc (16)" — the agent then silently exits code 0 without
send/hibernate, creating a daemon retry death spiral.

Run this check from the chamber directory:

```sh
for f in "../AGENTS.md" "../Makefile"; do
  if [ -f "$f" ] && ls -ldO "$f" 2>/dev/null | grep -q "dataless"; then
    echo "iCloud-evicted: $f — forcing download..."
    brctl download "$f" 2>/dev/null
    sleep 2
  fi
done
```

If any file remains unreadable after the download attempt (verify with
`cat "../AGENTS.md" > /dev/null 2>&1`), DO NOT silently exit. Instead:

1. Log the failure in `NOTES.md` under `## Friction log`.
2. `cryo-agent send "pre-flight failed: <file> iCloud-evicted and brctl download could not recover it"`
3. `cryo-agent hibernate --exit 1 --summary "pre-flight iCloud check failed; <file> unreadable"`
4. The daemon will retry. Do NOT skip the check — silent exits are
   what caused 12-attempt death spirals before this check existed.

If the check passes, proceed to Step 1.

### Step 1 — Orient

The wake prompt already includes `## Current Time`, `## Task`, `## TODO List`,
`## Inbox`, and (sometimes) `## System Notice`. Read those first. Then read
`NOTES.md` for durable cross-session context: project facts, open questions,
plans in flight, and friction that is not already recoverable from logs.

If `## System Notice` is present (delayed wake — the machine was suspended past
the scheduled time), proceed normally — a late weekly digest is still useful.
If delayed wakes become a pattern or affect the cadence, add a concise note
under `## Friction log`; otherwise rely on `cryo.log` for routine timing history.

**Branch decision.** Before going further, classify this wake using the
prompt's pre-rendered sections — only one branch should run per session:

| Trigger | How to detect | Goes to |
|---|---|---|
| Operator message | `## Inbox` has unclaimed messages | **Step 1.6** (inbox-reply) |
| First wake ever | No pending or claimed weekly advisor TODO in `## TODO List` | **Step 1.5** (bootstrap) |
| Monday 09:48 TODO fired | Claimed `[~]` weekly advisor TODO in `## TODO List` AND today is Monday in Asia/Shanghai | **Steps 2–6** (advisor scan) |
| Off-cycle TODO wake (rare) | Claimed `[~]` weekly advisor TODO but today is **not** Monday | Skip Steps 2–4; do Step 3 durable-context check + Step 4 outbox + Step 5 reschedule + Step 6 hibernate |

If multiple triggers match (e.g. operator messaged on Monday morning before
TODO firing), the inbox branch wins — handle the message, then let the next
TODO fire normally.

### Step 1.5 — Bootstrap branch (first session only)

Before running the advisor workflow, check whether this is the **bootstrap session** —
the first wake after `cryo start` or after the chamber was cancelled and
restarted. Detection: `## TODO List` has no pending or claimed weekly advisor
TODO. Do not use `NOTES.md` as a run log; it is durable memory only.

If no weekly advisor TODO exists, this is bootstrap. Skip Steps 2–4 and run
this abbreviated flow:

1. Leave `NOTES.md` unchanged unless you learn a durable fact, open question,
   in-flight plan, or friction item that belongs in the current NOTES template.

2. Send a brief outbox message:

   ```
   cryo-agent send "many-body-advisor chamber initialized; first weekly scan scheduled for Monday <YYYY-MM-DD> 09:48 Asia/Shanghai. No advisor workflow run this session."
   ```

3. Skip directly to Step 5 (schedule next Monday 09:48) and Step 6 (hibernate).
   Do **not** run the advisor workflow in the bootstrap
   session. Goal: avoid posting to Zulip on a non-Monday just because
   `cryo start` happened to fire mid-week.

If a weekly advisor TODO exists, this is a normal recurring chamber wake;
proceed to Step 2 when the Monday TODO is claimed, or follow the off-cycle
branch when it is not Monday.

### Step 1.6 — Inbox-driven branch (operator nudged the chamber)

If `## Inbox` in the wake prompt shows pending messages, the operator
invoked `cryo send "<msg>"` to nudge the chamber between Mondays. Do **not**
run the advisor workflow unless the operator explicitly asks for an ad-hoc
scan — the normal workflow is only for Monday-09:48 TODO firings. Instead:

1. **Receive** the current batch:

   ```
   cryo-agent receive
   ```

   This claims and archives the messages so they won't re-trigger next time.

2. **Decide what the operator wants.** Common patterns:

   - *Status check* ("are you alive?", "next wake?") — confirm the next
     Monday TODO is still pending and answer briefly.
   - *Ad-hoc advisor run* ("run the scan now", "rerun for the past 3
     days") — run the Step 2 advisor workflow end-to-end, then reply with what was posted. Note in the reply
     that the regular Monday scan will still fire as scheduled.
   - *Configuration question* (about the plan, NOTES.md, the schedule) —
     answer from `plan.md` / `NOTES.md` / `cryo-agent todo list`.
   - *Anything else* — answer plainly. If you genuinely don't know, say
     so; don't fabricate.

3. **Reply** via `cryo-agent send "<reply>"`. Draft the reply prose inline
   (Generation policy, top of this section), grounded in the operator's
   message and any tool outputs you gathered.

4. **Update** `NOTES.md` only if this reply reveals a durable fact, open
   question, in-flight plan, or friction item. Do not append a routine
   chronological run log; outbox messages and `cryo.log` already cover that.

5. **Verify** the Monday TODO is still pending via `cryo-agent todo list`.
   If TODO #1 (or its successor) is gone or its `at` time has drifted away
   from the next Monday-09:48 Asia/Shanghai, re-add one per Step 5.

6. **Hibernate** per Step 6. Use the inbox-reply summary, e.g.:

   ```
   cryo-agent hibernate --summary "inbox reply to operator; next Monday TODO intact"
   ```

   Never use `--complete` from this branch — the chamber is still active.

### Step 2 — Run the local advisor workflow end-to-end

Run the local workflow below. Do not invoke `Skill(...)`, do not read a plugin
or skill file from outside this project, and do not fetch workflow instructions
from the network. This chamber is configured for unattended `opencode`, so the
weekly path must be fully self-contained.

The project-side context the workflow needs (`.knowledge/INDEX.md`, watched
arXiv queries / authors / groups / repos under "Reliable update sources", and
the `make zulip-pull` / `make zulip-send` / `make zulip-topics` /
`make zulip-messages` Makefile targets) is in `../AGENTS.md`, the canonical
harness context. Draft any prose inline per the
Generation policy at the top of this section.

**Important:** even if the workflow is partially blocked or you cannot
complete the full flow, you MUST still send a status outbox message
(`cryo-agent send "..."`) and call `cryo-agent hibernate` before exiting.
Silent exits violate the chamber protocol and force the daemon to write a
generic fallback reply.

The workflow runs 6 steps:

1. **Sync Zulip** — `make zulip-pull` from `..` (project root).
2. **Build the week view** — read messages whose `timestamp` falls in the last
   7 calendar days ending today.
3. **Audit TODOs** — for the recurring participants in the `ManyBodyHarness`
   stream (no fixed roster is hardcoded; identify active collaborators from
   the synced Zulip week view and `../AGENTS.md`), classify each as
   TODOs-respected / at-risk / missing-next-week-TODO. If no active
   collaborator evidence is available, say so explicitly instead of asking.
4. **Search current reliable sources** — uses the
   "Reliable update sources" section in `../AGENTS.md`
   (arXiv keywords, watched authors, watched repos, allow/avoid lists).
   Include only papers from the **last 7 days**, plus optionally a key paper
   up to **3 years** old that ties to a topic discussed recently in the
   stream — mixable, at most three total. Deduplicate against
   `.knowledge/INDEX.md` and prior advisor posts in the `weekly advisor`
   Zulip topic — do not repeat a paper or release already mentioned.
5. **Draft the advisor post** — create `../.zulip/.drafts/` if needed, then
   write `../.zulip/.drafts/weekly-advisor-YYYY-MM-DD.md`. Minimalist: a
   header line `weekly advisor · YYYY-MM-DD → YYYY-MM-DD`, then two blocks
   only — (1) `TODOs` as status-tagged bullets
   `- [In progress] @owner — task` (statuses Done / In progress / At risk /
   New), and (2) `Key reads (≤3)` as `- [Title](url) (date) — why it
   matters`. No greeting paragraph, no goal-audit table, no
   suggestions/recommendations section. Drop the Key reads block entirely if
   nothing qualifies.
6. **Review and send** — verifies the `weekly advisor` topic exists, then
   `make zulip-send TOPIC="weekly advisor" MSG_FILE=...`, then
   `make zulip-pull` to mirror back.

**Auto-approval (this chamber runs autonomously):**

The local workflow's Step 7 says "Show the draft to the user first. Do not
send until the user explicitly approves." In this chamber the agent IS effectively the
operator — there is no human in the session loop. So:

- The chamber-running agent acts as the approver. After Step 5 produces the
  draft, the agent should self-review: every active collaborator has a TODO
  status, 0–3 key reads included (last 7 days, or a ≤3-year paper tied to a
  recently discussed topic; deduplicated), no AI signature. Then proceed to
  Step 6 without prompting.
- **Sanity-check before sending.** If the draft is structurally broken
  (empty, contains visible placeholder text like `<student>` / `<date>` /
  `TBD`, or fails to reference any specific Zulip permalink), do **not**
  send.
  Instead leave the draft on disk, write a `cryo-agent send` outbox note
  describing the failure, exit `1` so the daemon retries, and skip the
  Monday reschedule (it'll happen via retry → next-Monday on a successful
  run).

**Pause-prone points already pre-resolved by the project's `AGENTS.md`:**

- *Roster*: `ManyBodyHarness` has no hardcoded roster. Identify active
  collaborators from the synced Zulip week view; if evidence is thin, write a
  "no active collaborator evidence found this week" audit line rather than
  stalling.
- *Reliable-source config*: `../AGENTS.md` has the
  "Reliable update sources" section. The workflow should use it without asking.

If the workflow nevertheless reaches a point where ordinary interactive use
would ask for user input, the agent should answer using the information in
`../AGENTS.md` and proceed. Do not stall the session waiting for human input.

If the workflow fails for an external reason (Zulip auth broken, network down,
arXiv rate limit), let the session exit `1` — the daemon will retry on its
own backoff schedule.

### Step 3 — Record Durable Context

Update `NOTES.md` only for durable facts, open questions / hypotheses,
plans in flight, or friction that future sessions need and cannot recover
from `cryo.log`, outbox messages, or repository files. Do not append routine
per-session run logs. Trim stale notes if the file grows past ~200 lines.

### Step 4 — Send chamber outbox message (required by protocol)

```
cryo-agent send "weekly advisor: posted digest to Zulip 'weekly advisor' topic; <N> key reads flagged"
```

This is the cryochamber-protocol-required outbox message. It accumulates
locally in `messages/outbox/` since no sync channel is configured. It is
**not** the digest — the digest is posted to Zulip directly by the workflow in
its Step 7. Use this outbox line as the chamber's own per-run status: cite the
draft path under `../.zulip/.drafts/` and a one-line summary of what was
posted. If sanity-check rejected the draft (see Step 2 workflow), the outbox
message should explain what was wrong instead.

Draft this one-liner inline (Generation policy, top of this section), using
the workflow outcome — status, draft path, post URL or failure reason — to
write something crisp.

### Step 5 — Schedule next Monday 09:48 Asia/Shanghai

Compute the ISO8601 timestamp for the *next* Monday at 09:48 in Asia/Shanghai
(UTC+8). The agent should:

1. Get current time via `cryo-agent time`.
2. Reason about which day of the week it is and how many days until next
   Monday (1 if today is Sunday, 7 if today is Monday, etc. — always pick
   the *next* Monday, never today, even if it's Monday before 09:48, to
   avoid double-firing).
3. Construct an absolute timestamp as `YYYY-MM-DDTHH:MM` — local time, **no
   seconds and no timezone suffix** (a `:00` seconds field or a `+08:00`
   offset is rejected as "unrecognized time expression"), e.g.
   `2026-05-11T09:48`.
4. Validate by passing the same string back through
   `cryo-agent time "<the-timestamp>"` — it echoes the timestamp when the
   form is accepted, and errors on a seconds or `+08:00` form.
5. Schedule:

   ```
   cryo-agent todo add "Run weekly many-body advisor workflow" --at <ISO>
   ```

`cryo-agent time` does not parse natural language like "next Monday 9am" —
the agent must compute the absolute timestamp itself.

### Step 6 — Hibernate (must be the final tool call)

```
cryo-agent hibernate --summary "weekly advisor scan; next: Mon 09:48 Asia/Shanghai"
```

If the workflow failed in Step 2 with a recoverable error (network blip,
transient Zulip 5xx), exit with `--exit 1` instead so the daemon retries
on its backoff schedule:

```
cryo-agent hibernate --exit 1 --summary "advisor workflow failed: <reason>; retry"
```

Never use `--complete` — this plan is a recurring weekly task with no
terminal condition.

## Configuration

- **Chamber lives in `./.cryo/`** under the quantum.harness project root (`..` from here).
- **Agent: `opencode`** — invoked by the daemon as `opencode run "<wake prompt>"`.
  The Monday digest path is self-contained in this plan and uses project-local
  commands/files only; it must not depend on Claude Code plugin skills or
  external skill directories.
- **No sync channel** — outbox messages stay local; the digest is posted to
  Zulip directly by the local advisor workflow via the project's existing
  `make zulip-send` bridge.
- **`watch_inbox = true`** — mixed mode. Scheduled Monday-09:48 TODO drives
  the weekly advisor scan, AND `cryo send "<msg>"` from the operator wakes
  the agent for an inbox reply (handled by Step 1.6). Flip back to `false`
  + `cryo restart` if the chamber should ignore inbox traffic and be purely
  autonomous again.
## Notes

### State strategy

| Thing to remember | Where |
|---|---|
| Next wake (Monday 09:48) | `cryo-agent todo --at <ISO>` |
| Durable facts, hypotheses, plans, and friction | `NOTES.md` |
| Routine run history and delay history | `cryo.log` / outbox messages |
| arXiv listing cursor / GitHub `since` timestamps | Recomputed during each weekly workflow run |
| Already-saved papers | `.knowledge/INDEX.md` (the workflow reads it) |

### Working directory

The agent runs from `chamber_dir = .cryo/`. Commands that need the project's
Makefile (e.g. `make zulip-send`) must be run from `..`. The local advisor
workflow uses this convention.

### Time computation

`cryo-agent time` accepts:
- (no arg) → prints current time as ISO8601
- `+N minutes|hours|days|weeks` → relative offset
- absolute `YYYY-MM-DDTHH:MM` (local time, no seconds, no `+08:00` offset) → pass-through validation; forms with seconds or an offset are rejected

It does **not** parse "next Monday 9am" or other natural-language times. The
agent must compute Monday-09:48 absolute timestamps itself (see Step 5).

### Failure handling

- Workflow failure (transient): exit `1`. The daemon re-queues the missed TODO
  with an `(attempt k)` suffix and `2^k`-minute backoff (cap 1 day), visible
  via `cryo-agent todo list`.
- Workflow failure (persistent, e.g. Zulip auth broken): the daemon will keep
  retrying weekly. The human can run `cryo log` / `cryo watch` to inspect.
- External plugin availability is irrelevant to the chamber runtime. If a
  future edit reintroduces `Skill(...)` or reads from an external skill path,
  treat that as a regression for unattended `opencode`.

### Provider configuration

Currently using a single Claude provider via the user's normal config. To
pin a specific provider profile, edit `cryo.toml`:

```toml
[provider]
name = "opencode"
env = { ANTHROPIC_API_KEY = "sk-ant-..." }
```

Multi-provider rotation has been removed in cryochamber v0.2.x; only one
`[provider]` table is honoured. Add `cryo.toml` to `.gitignore` if it ever
contains secrets.
