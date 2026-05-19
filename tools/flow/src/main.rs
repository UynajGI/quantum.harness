use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, String>;

const GATE_PENDING: &str = "pending";
const GATE_PASSED: &str = "passed";
const GATE_FAILED: &str = "failed";
const FLOW_TEMPLATE_FILE: &str = "flow.toml";
const PROTOCOL_FILE: &str = "protocol.toml";
const ACTOR_ENV: &str = "FLOW_ACTOR_ID";

// Identity is unforgeable-by-label: either host-injected via FLOW_ACTOR_ID,
// or derived from the parent process id. Different subagent processes get
// different PPIDs naturally; the same agent across calls keeps the same PPID.
fn current_identity() -> String {
    env::var(ACTOR_ENV)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("ppid:{}", std::os::unix::process::parent_id()))
}

fn stamped(args: &[String], default_label: &str) -> Actor {
    let label = option_value(args, "--actor").unwrap_or_else(|| default_label.to_string());
    Actor {
        label,
        identity: current_identity(),
    }
}

const ATTEMPT_AUDIT: &str = "audit";

// ─── Domain types (also the on-disk shape of state.toml) ────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Actor {
    #[serde(rename = "actor_label")]
    label: String,
    #[serde(rename = "actor_identity")]
    identity: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Gate {
    #[serde(default)]
    requires: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Report {
    path: String,
    hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RunResult {
    ok: bool,
    output: String,
    at: String,
}

// One audit attempt's verdict on one claim. Written by the audit subagent
// into a verify_*.toml sidecar, parsed at attempt-finish, exposed via
// `flow status --json`. Typed status rejects sidecar typos at parse time.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum VerdictStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Verdict {
    claim: String,
    status: VerdictStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Attempt {
    seq: u64,
    gate: String,
    kind: String,
    #[serde(flatten)]
    actor: Actor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    executor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    finished: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    report: Option<Report>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    verdicts: Vec<Verdict>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Artifact {
    path: String,
    kind: String,
    hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    producer: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    deps: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Override {
    check: String,
    gate: String,
    reason: String,
    #[serde(flatten)]
    actor: Actor,
    at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Decision {
    id: String,
    question: String,
    choice: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(flatten)]
    actor: Actor,
    at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Deviation {
    id: String,
    statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(flatten)]
    actor: Actor,
    at: String,
}

#[derive(Debug, Default)]
struct State {
    seq: u64,
    flow_id: Option<String>,
    gates: BTreeMap<String, Gate>,
    attempts: BTreeMap<String, Attempt>,
    artifacts: BTreeMap<String, Artifact>,
    overrides: Vec<Override>,
    decisions: Vec<Decision>,
    deviations: Vec<Deviation>,
    run_results: BTreeMap<String, RunResult>,
    children: Vec<String>,
}

struct Ctx<'a> {
    dir: &'a Path,
    state: &'a State,
    protocol: &'a Protocol,
}

// ─── Events (typed append-only log) ──────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
enum Event {
    #[serde(rename = "flow_initialized")]
    FlowInitialized { flow_id: String, created_at: String },
    #[serde(rename = "gate_added")]
    GateAdded {
        id: String,
        #[serde(default)]
        requires: Vec<String>,
    },
    #[serde(rename = "attempt_started")]
    AttemptStarted {
        id: String,
        gate: String,
        kind: String,
        #[serde(flatten)]
        actor: Actor,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        command: Option<String>,
    },
    #[serde(rename = "attempt_finished")]
    AttemptFinished {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        report: Option<Report>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        verdicts: Vec<Verdict>,
    },
    #[serde(rename = "run_recorded")]
    RunRecorded {
        check: String,
        #[serde(flatten)]
        result: RunResult,
    },
    #[serde(rename = "artifact_added")]
    ArtifactAdded {
        id: String,
        path: String,
        kind: String,
        hash: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        producer: Option<String>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        deps: BTreeMap<String, String>,
    },
    #[serde(rename = "override_recorded")]
    OverrideRecorded {
        check: String,
        gate: String,
        reason: String,
        #[serde(flatten)]
        actor: Actor,
        at: String,
    },
    #[serde(rename = "decision_recorded")]
    DecisionRecorded {
        id: String,
        question: String,
        choice: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        #[serde(flatten)]
        actor: Actor,
        at: String,
    },
    #[serde(rename = "deviation_recorded")]
    DeviationRecorded {
        id: String,
        statement: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        #[serde(flatten)]
        actor: Actor,
        at: String,
    },
    #[serde(rename = "child_attached")]
    ChildAttached { path: String },
}

// ─── Protocol (the contract) ─────────────────────────────────────────────────

#[derive(Deserialize, Clone, Debug)]
struct GateSpec {
    id: String,
    #[serde(default)]
    requires: Vec<String>,
}

#[derive(Deserialize)]
struct FlowTemplate {
    flow: Option<FlowTemplateHeader>,
    #[serde(default)]
    gates: Vec<GateSpec>,
}

#[derive(Deserialize)]
struct FlowTemplateHeader {
    id: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct Check {
    id: String,
    kind: CheckKind,
    gate: String,
    #[serde(default)]
    cmd: Option<String>,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    fields: Vec<String>,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    against: Vec<String>,
    #[serde(default)]
    compare: Vec<Compare>,
}

// One-word generic check kinds. Each names what the check does, never what
// artifact it touches. Domain semantics live in protocol.toml fields.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CheckKind {
    Audit,  // verifier ran with a distinct actor
    Run,    // external command exits zero
    Exists, // declared fields/paths are present
    Agree,  // declared values match across sources
    Near,   // numeric within tolerance of reference
    Fresh,  // artifact and declared sources unchanged since registration
    Cover,  // observed path set exactly matches declared paths
}

#[derive(Deserialize, Clone, Debug)]
struct Compare {
    actual: ValueRef,
    reference: ValueRef,
    #[serde(default)]
    uncertainty: Option<ValueRef>,
    tolerance: Tolerance,
}

#[derive(Deserialize, Clone, Debug)]
struct ValueRef {
    path: String,
    field: String,
}

#[derive(Deserialize, Clone, Debug)]
struct Tolerance {
    #[serde(default)]
    abs: Option<f64>,
    #[serde(default)]
    rel: Option<f64>,
    #[serde(default)]
    sigma: Option<f64>,
}

#[derive(Deserialize, Clone, Debug, Default)]
struct ProtocolClaim {
    #[serde(default)]
    id: String,
    #[serde(default)]
    statement: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
struct ProtocolDeviation {
    #[serde(default)]
    id: String,
    #[serde(default)]
    statement: String,
    #[serde(default)]
    reason: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
struct ProtocolPending {
    #[serde(default)]
    id: String,
    #[serde(default)]
    statement: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    reason: String,
}

#[derive(Deserialize, Default)]
struct Protocol {
    #[serde(default, rename = "claims")]
    claims: Vec<ProtocolClaim>,
    #[serde(default, rename = "checks")]
    checks: Vec<Check>,
    #[serde(default, rename = "deviations")]
    deviations: Vec<ProtocolDeviation>,
    #[serde(default, rename = "pending")]
    pending: Vec<ProtocolPending>,
}

// Audit subagents write a verify_*.toml sidecar next to their markdown report.
// Flow parses it at attempt-finish and bakes verdicts into the event log so
// `flow status --json` can derive per-claim chip status without grepping prose.
#[derive(Deserialize)]
struct VerifyDoc {
    #[serde(default)]
    verdicts: Vec<Verdict>,
}

#[derive(Clone, Debug, Serialize)]
struct CheckResult {
    id: String,
    pass: bool,
    detail: String,
}

// ─── Entry ───────────────────────────────────────────────────────────────────

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(usage());
    }

    match args.remove(0).as_str() {
        "init" => cmd_init(&args),
        "status" => cmd_status(&args),
        "next" => cmd_next(&args),
        "require" => cmd_require(&args),
        "artifact" => cmd_artifact(&args),
        "attempt" => cmd_attempt(&args),
        "check" => cmd_check(&args),
        "override" => cmd_override(&args),
        "decide" => cmd_decide(&args),
        "deviate" => cmd_deviate(&args),
        "attach" => cmd_attach(&args),
        "help" | "--help" | "-h" => {
            println!("{}", usage());
            Ok(())
        }
        other => Err(format!("unknown command: {other}\n{}", usage())),
    }
}

fn usage() -> String {
    "usage: harness-flow <init|status|next|require|artifact|attempt|check|override|decide|deviate|attach> ..."
        .to_string()
}

// ─── init ────────────────────────────────────────────────────────────────────

fn cmd_init(args: &[String]) -> Result<()> {
    if args.len() != 3 || args[1] != "--template" {
        return Err("usage: harness-flow init <dir> --template <template.toml>".to_string());
    }
    let dir = Path::new(&args[0]);
    let template = Path::new(&args[2]);
    let template_text = fs::read_to_string(template).map_err(|e| e.to_string())?;
    let (flow_id, gates) = parse_template_text(&template_text)?;

    fs::create_dir_all(progress_dir(dir)).map_err(|e| e.to_string())?;
    with_flow_lock(dir, || {
        let events = events_path(dir);
        if events.exists() {
            return Err(format!("flow already exists: {}", dir.display()));
        }
        persist_flow_template(dir, &template_text)?;

        append_event(
            dir,
            &Event::FlowInitialized {
                flow_id: flow_id.unwrap_or_else(|| flow_id_from_path(dir)),
                created_at: now_id(),
            },
        )?;
        for gate in gates {
            append_event(
                dir,
                &Event::GateAdded {
                    id: gate.id,
                    requires: gate.requires,
                },
            )?;
        }
        rebuild(dir)?;
        Ok(())
    })
}

// ─── status ──────────────────────────────────────────────────────────────────

fn cmd_status(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err("usage: harness-flow status <dir> [--full | --recursive | --json]".to_string());
    }
    let dir = Path::new(&args[0]);
    let flags: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    let state = with_flow_lock(dir, || rebuild(dir))?;
    let protocol = load_protocol(dir)?;
    let ctx = Ctx {
        dir,
        state: &state,
        protocol: &protocol,
    };
    if flags.contains(&"--json") {
        print_status_json(&ctx)
    } else if flags.contains(&"--full") || flags.contains(&"--recursive") {
        print_status(&ctx, 0, flags.contains(&"--recursive"))
    } else {
        print_status_terse(&ctx)
    }
}

fn print_status_terse(ctx: &Ctx) -> Result<()> {
    let label = ctx
        .state
        .flow_id
        .as_deref()
        .unwrap_or_else(|| ctx.dir.file_name().and_then(|v| v.to_str()).unwrap_or("."));
    println!("flow {label}");

    let gates = evaluate_all(ctx);
    let current = gates
        .iter()
        .find(|g| g.runnable && g.status != GATE_PASSED)
        .or_else(|| gates.iter().find(|g| g.status == GATE_FAILED))
        .or_else(|| gates.iter().find(|g| g.status != GATE_PASSED))
        .or_else(|| gates.last());

    let Some(gate) = current else {
        println!("gate none");
        println!("next none");
        return Ok(());
    };

    println!("gate {} {}", gate.id, gate.status);
    if gate.status != GATE_PASSED {
        if let Some(failed) = gate.checks.iter().find(|c| !c.pass) {
            println!("blocker {}: {}", failed.id, failed.detail);
        } else if gate.status == GATE_PENDING {
            println!("blocker no finished attempt");
        }
    }

    let deviations = ctx
        .protocol
        .deviations
        .iter()
        .filter(|d| !d.id.is_empty())
        .count()
        + ctx.state.deviations.len();
    let decisions = ctx.state.decisions.len();
    let overrides = ctx.state.overrides.len();
    let pending = ctx
        .protocol
        .pending
        .iter()
        .filter(|p| !p.id.is_empty())
        .count();
    if deviations + decisions + overrides + pending > 0 {
        println!(
            "flags deviations={deviations} decisions={decisions} overrides={overrides} pending={pending}"
        );
    }

    if gate.runnable && gate.status != GATE_PASSED {
        println!(
            "next flow attempt start {} {} --kind <kind> --actor agent:<role>",
            ctx.dir.display(),
            gate.id
        );
    } else {
        let next: Vec<&str> = gates.iter().filter(|g| g.runnable).map(|g| g.id).collect();
        if let Some(next_gate) = next.first() {
            println!(
                "next flow attempt start {} {} --kind <kind> --actor agent:<role>",
                ctx.dir.display(),
                next_gate
            );
        } else {
            println!("next none");
        }
    }
    Ok(())
}

fn print_status(ctx: &Ctx, indent: usize, recursive: bool) -> Result<()> {
    let pad = " ".repeat(indent);
    let label = ctx
        .state
        .flow_id
        .as_deref()
        .unwrap_or_else(|| ctx.dir.file_name().and_then(|v| v.to_str()).unwrap_or("."));
    println!("{pad}flow {label}");

    let gates = evaluate_all(ctx);
    let mut next_marked = false;
    for g in &gates {
        let mut suffix = String::new();
        let n_over = ctx
            .state
            .overrides
            .iter()
            .filter(|o| o.gate == g.id)
            .count();
        if n_over > 0 {
            suffix.push_str(&format!("\t⊘ {n_over}"));
        }
        if g.runnable && !next_marked {
            next_marked = true;
            suffix.push_str("\t▶");
        }
        println!("{pad}{}\t{}{suffix}", g.id, g.status);
    }

    let declared_deviations: Vec<&ProtocolDeviation> = ctx
        .protocol
        .deviations
        .iter()
        .filter(|d| !d.id.is_empty())
        .collect();
    let total_deviations = declared_deviations.len() + ctx.state.deviations.len();
    if total_deviations > 0 {
        println!("{pad}⚠ deviations ({total_deviations})");
        for d in &declared_deviations {
            print_deviation(&pad, &d.id, &d.statement, Some(&d.reason));
        }
        for d in &ctx.state.deviations {
            print_deviation(&pad, &d.id, &d.statement, d.reason.as_deref());
        }
    }

    if !ctx.state.overrides.is_empty() {
        println!("{pad}⊘ overrides ({})", ctx.state.overrides.len());
        for o in &ctx.state.overrides {
            println!("{pad}  ⊘ {} ({}) — {}", o.check, o.gate, o.reason);
        }
    }

    if !ctx.state.decisions.is_empty() {
        println!("{pad}decisions ({})", ctx.state.decisions.len());
        for d in &ctx.state.decisions {
            println!("{pad}  · {} — {} = {}", d.id, d.question, d.choice);
        }
    }

    let pending: Vec<&ProtocolPending> = ctx
        .protocol
        .pending
        .iter()
        .filter(|p| !p.id.is_empty())
        .collect();
    if !pending.is_empty() {
        println!("{pad}pending ({})", pending.len());
        for p in pending {
            if p.reason.is_empty() {
                println!("{pad}  · {} — {}", p.id, p.statement);
            } else {
                println!("{pad}  · {} — {} ({})", p.id, p.statement, p.reason);
            }
        }
    }

    let next: Vec<&str> = gates.iter().filter(|g| g.runnable).map(|g| g.id).collect();
    if !next.is_empty() {
        let dir_arg = ctx.dir.display().to_string();
        println!("{pad}next");
        for gate in next {
            println!(
                "{pad}  flow attempt start {dir_arg} {gate} --kind <kind> --actor agent:<role>"
            );
        }
    }

    if recursive && !ctx.state.children.is_empty() {
        println!("{pad}children");
        for child in &ctx.state.children {
            println!("{pad}- {child}");
            let child_dir = Path::new(child);
            let child_state = with_flow_lock(child_dir, || rebuild(child_dir))?;
            let child_protocol = load_protocol(child_dir)?;
            print_status(
                &Ctx {
                    dir: child_dir,
                    state: &child_state,
                    protocol: &child_protocol,
                },
                indent + 2,
                true,
            )?;
        }
    }
    Ok(())
}

// JSON read API for consumers (render.py, hooks, dashboards). Gate evaluation
// is computed once in DAG order so requirements_passed reads from the cache
// instead of re-entering evaluate_gate per requirement.
#[derive(Serialize)]
struct GateEval<'a> {
    id: &'a str,
    status: String,
    requires: &'a [String],
    runnable: bool,
    checks: Vec<CheckResult>,
}

#[derive(Serialize)]
struct StatusJson<'a> {
    flow_id: &'a str,
    gates: &'a [GateEval<'a>],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    claims: Vec<ClaimJson<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    deviations: Vec<DeviationJson<'a>>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    decisions: &'a [Decision],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    overrides: &'a [Override],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pending: Vec<&'a ProtocolPending>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    next: Vec<&'a str>,
}

#[derive(Serialize)]
struct ClaimJson<'a> {
    id: &'a str,
    statement: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    verdict: Option<VerdictStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<&'a str>,
}

#[derive(Serialize)]
struct DeviationJson<'a> {
    id: &'a str,
    statement: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
    source: &'static str,
}

fn evaluate_all<'a>(ctx: &Ctx<'a>) -> Vec<GateEval<'a>> {
    let order = dag_order(ctx.state);
    let mut out: Vec<GateEval> = Vec::with_capacity(order.len());
    let mut idx: BTreeMap<&str, usize> = BTreeMap::new();
    for gate in order {
        let (status, checks) = evaluate_gate(ctx, gate);
        let spec = &ctx.state.gates[gate];
        let requires_ok = spec.requires.iter().all(|r| {
            idx.get(r.as_str())
                .map(|i| out[*i].status == GATE_PASSED)
                .unwrap_or(false)
        });
        let runnable = status != GATE_PASSED && requires_ok;
        idx.insert(gate, out.len());
        out.push(GateEval {
            id: gate,
            status,
            requires: &spec.requires,
            runnable,
            checks,
        });
    }
    out
}

fn print_status_json(ctx: &Ctx) -> Result<()> {
    let gates = evaluate_all(ctx);
    let next: Vec<&str> = gates.iter().filter(|g| g.runnable).map(|g| g.id).collect();
    let claims: Vec<ClaimJson> = ctx
        .protocol
        .claims
        .iter()
        .filter(|c| !c.id.is_empty())
        .map(|c| {
            let v = claim_verdict(ctx.state, &c.id);
            ClaimJson {
                id: &c.id,
                statement: &c.statement,
                verdict: v.map(|v| v.status),
                note: v.and_then(|v| v.note.as_deref()),
            }
        })
        .collect();
    let deviations: Vec<DeviationJson> = ctx
        .protocol
        .deviations
        .iter()
        .filter(|d| !d.id.is_empty())
        .map(|d| DeviationJson {
            id: &d.id,
            statement: &d.statement,
            reason: (!d.reason.is_empty()).then_some(d.reason.as_str()),
            source: "declared",
        })
        .chain(ctx.state.deviations.iter().map(|d| DeviationJson {
            id: &d.id,
            statement: &d.statement,
            reason: d.reason.as_deref(),
            source: "recorded",
        }))
        .collect();
    let pending: Vec<&ProtocolPending> = ctx
        .protocol
        .pending
        .iter()
        .filter(|p| !p.id.is_empty())
        .collect();
    let out = StatusJson {
        flow_id: ctx.state.flow_id.as_deref().unwrap_or(""),
        gates: &gates,
        claims,
        deviations,
        decisions: &ctx.state.decisions,
        overrides: &ctx.state.overrides,
        pending,
        next,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?
    );
    Ok(())
}

// ─── next ────────────────────────────────────────────────────────────────────

fn cmd_next(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        return Err("usage: harness-flow next <dir>".to_string());
    }
    let dir = Path::new(&args[0]);
    let dir_arg = &args[0];
    let state = with_flow_lock(dir, || rebuild(dir))?;
    let protocol = load_protocol(dir)?;
    for gate in ready_gates(&Ctx {
        dir,
        state: &state,
        protocol: &protocol,
    }) {
        println!("{gate}");
        println!("  flow attempt start {dir_arg} {gate} --kind <kind> --actor agent:<role>");
    }
    Ok(())
}

// ─── require ─────────────────────────────────────────────────────────────────

fn cmd_require(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        return Err("usage: harness-flow require <dir> <gate>".to_string());
    }
    let dir = Path::new(&args[0]);
    let gate = &args[1];
    let state = with_flow_lock(dir, || rebuild(dir))?;
    let protocol = load_protocol(dir)?;
    if gate_passed(
        &Ctx {
            dir,
            state: &state,
            protocol: &protocol,
        },
        gate,
    ) {
        Ok(())
    } else {
        Err(format!("{gate} not passed"))
    }
}

// ─── artifact ────────────────────────────────────────────────────────────────

fn cmd_artifact(args: &[String]) -> Result<()> {
    if args.len() < 5 || args[0] != "add" {
        return Err(
            "usage: harness-flow artifact add <dir> <id> <path> --kind <kind> [--producer <attempt>]"
                .to_string(),
        );
    }
    let dir = Path::new(&args[1]);
    let id = args[2].clone();
    let path = Path::new(&args[3]);
    let kind = required_option(args, "--kind")?;
    let producer = option_value(args, "--producer");
    let hash = file_hash(path)?;
    let rel = path
        .strip_prefix(dir)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());
    let deps = snapshot_deps(dir, &rel)?;
    with_flow_lock(dir, || {
        ensure_flow(dir)?;
        let state = rebuild(dir)?;
        if let Some(producer) = &producer {
            if !state.attempts.contains_key(producer) {
                return Err(format!("unknown producer attempt: {producer}"));
            }
        }
        append_event(
            dir,
            &Event::ArtifactAdded {
                id: id.clone(),
                path: rel,
                kind,
                hash,
                producer: producer.clone(),
                deps,
            },
        )?;
        rebuild(dir)?;
        Ok(())
    })
}

// ─── attempt ─────────────────────────────────────────────────────────────────

fn cmd_attempt(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err("usage: harness-flow attempt <start|finish> ...".to_string());
    }
    match args[0].as_str() {
        "start" => cmd_attempt_start(args),
        "finish" => cmd_attempt_finish(args),
        _ => Err("usage: harness-flow attempt <start|finish> ...".to_string()),
    }
}

fn cmd_attempt_start(args: &[String]) -> Result<()> {
    if args.len() < 6 {
        return Err(
            "usage: harness-flow attempt start <dir> <gate> --kind <kind> --actor <actor> [--executor <exec>] [--command <cmd>]"
                .to_string(),
        );
    }
    let dir = Path::new(&args[1]);
    let gate = args[2].clone();
    let kind = required_option(args, "--kind")?;
    required_option(args, "--actor")?;
    let actor = stamped(args, "");
    let executor = option_value(args, "--executor");
    let command = option_value(args, "--command");
    with_flow_lock(dir, || {
        let state = rebuild(dir)?;
        let protocol = load_protocol(dir)?;
        if !state.gates.contains_key(&gate) {
            return Err(format!("unknown gate: {gate}"));
        }
        if !requirements_passed(
            &Ctx {
                dir,
                state: &state,
                protocol: &protocol,
            },
            &gate,
        ) {
            return Err(format!("{gate} requirements not passed"));
        }
        let id = format!("a{}", now_id());
        append_event(
            dir,
            &Event::AttemptStarted {
                id: id.clone(),
                gate,
                kind,
                actor,
                executor,
                command,
            },
        )?;
        rebuild(dir)?;
        println!("{id}");
        Ok(())
    })
}

fn cmd_attempt_finish(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        return Err(
            "usage: harness-flow attempt finish <dir> <attempt> [--report <path>]".to_string(),
        );
    }
    let dir = Path::new(&args[1]);
    let id = args[2].clone();
    let (report, verdicts) = match option_value(args, "--report") {
        Some(raw) => {
            let path = Path::new(&raw);
            if !path.exists() {
                return Err(format!("report not found: {raw}"));
            }
            let hash = file_hash(path)?;
            let sidecar = path.with_extension("toml");
            let verdicts = if sidecar.exists() && sidecar != path {
                let text = fs::read_to_string(&sidecar).map_err(|e| e.to_string())?;
                toml::from_str::<VerifyDoc>(&text)
                    .map_err(|e| format!("verify sidecar {}: {e}", sidecar.display()))?
                    .verdicts
            } else {
                Vec::new()
            };
            let rel = path
                .strip_prefix(dir)
                .map(|p| p.display().to_string())
                .unwrap_or(raw);
            (Some(Report { path: rel, hash }), verdicts)
        }
        None => (None, Vec::new()),
    };
    with_flow_lock(dir, || {
        let state = rebuild(dir)?;
        let attempt = state
            .attempts
            .get(&id)
            .ok_or_else(|| format!("unknown attempt: {id}"))?
            .clone();
        if attempt.finished {
            return Err(format!("attempt already finished: {id}"));
        }
        append_event(
            dir,
            &Event::AttemptFinished {
                id: id.clone(),
                report,
                verdicts,
            },
        )?;
        // Run-kind checks fire here so `flow status` stays pure.
        let protocol = load_protocol(dir)?;
        for check in &protocol.checks {
            if check.gate != attempt.gate || check.kind != CheckKind::Run {
                continue;
            }
            let result = run_check_command(dir, check);
            append_event(
                dir,
                &Event::RunRecorded {
                    check: check.id.clone(),
                    result,
                },
            )?;
        }
        let state = rebuild(dir)?;
        let (status, results) = evaluate_gate(
            &Ctx {
                dir,
                state: &state,
                protocol: &protocol,
            },
            &attempt.gate,
        );
        for r in &results {
            let mark = if r.pass { "ok" } else { "fail" };
            println!("{mark}\t{}\t{}", r.id, r.detail);
        }
        println!("status\t{status}");
        Ok(())
    })
}

// ─── check ───────────────────────────────────────────────────────────────────

fn cmd_check(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        return Err("usage: harness-flow check <dir> <gate>".to_string());
    }
    let dir = Path::new(&args[0]);
    let gate = &args[1];
    let state = with_flow_lock(dir, || rebuild(dir))?;
    let protocol = load_protocol(dir)?;
    if !state.gates.contains_key(gate) {
        return Err(format!("unknown gate: {gate}"));
    }
    let (status, results) = evaluate_gate(
        &Ctx {
            dir,
            state: &state,
            protocol: &protocol,
        },
        gate,
    );
    for r in &results {
        let mark = if r.pass { "ok" } else { "fail" };
        println!("{mark}\t{}\t{}", r.id, r.detail);
    }
    println!("status\t{status}");
    if status != GATE_PASSED {
        process::exit(1);
    }
    Ok(())
}

// ─── override ────────────────────────────────────────────────────────────────

// Agent invokes only after presenting the bypass via the host's option API
// and getting user confirmation; the CLI itself is non-interactive.
fn cmd_override(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        return Err(
            "usage: harness-flow override <dir> <check-id> --reason <text> [--actor <actor>]"
                .to_string(),
        );
    }
    let dir = Path::new(&args[0]);
    let check_id = args[1].clone();
    let reason = required_option(args, "--reason")?;
    let actor = stamped(args, "user");
    with_flow_lock(dir, || {
        let state = rebuild(dir)?;
        let protocol = load_protocol(dir)?;
        let check = protocol
            .checks
            .iter()
            .find(|c| c.id == check_id)
            .ok_or_else(|| format!("unknown check: {check_id}"))?;
        if !state.gates.contains_key(&check.gate) {
            return Err(format!(
                "check {check_id} targets unknown gate {}",
                check.gate
            ));
        }
        append_event(
            dir,
            &Event::OverrideRecorded {
                check: check_id.clone(),
                gate: check.gate.clone(),
                reason,
                actor,
                at: now_id(),
            },
        )?;
        rebuild(dir)?;
        println!("override recorded: {check_id} (gate {})", check.gate);
        Ok(())
    })
}

// ─── decide ──────────────────────────────────────────────────────────────────

fn cmd_decide(args: &[String]) -> Result<()> {
    if args.len() < 7 {
        return Err(
            "usage: harness-flow decide <dir> --id <id> --question <text> --choice <text> [--reason <text>] [--actor <actor>]"
                .to_string(),
        );
    }
    let dir = Path::new(&args[0]);
    let id = required_option(args, "--id")?;
    let question = required_option(args, "--question")?;
    let choice = required_option(args, "--choice")?;
    let reason = option_value(args, "--reason");
    let actor = stamped(args, "user");
    with_flow_lock(dir, || {
        ensure_flow(dir)?;
        append_event(
            dir,
            &Event::DecisionRecorded {
                id: id.clone(),
                question,
                choice,
                reason,
                actor,
                at: now_id(),
            },
        )?;
        rebuild(dir)?;
        println!("decision recorded: {id}");
        Ok(())
    })
}

// ─── deviate ─────────────────────────────────────────────────────────────────

fn cmd_deviate(args: &[String]) -> Result<()> {
    if args.len() < 5 {
        return Err(
            "usage: harness-flow deviate <dir> --id <id> --statement <text> [--reason <text>] [--actor <actor>]"
                .to_string(),
        );
    }
    let dir = Path::new(&args[0]);
    let id = required_option(args, "--id")?;
    let statement = required_option(args, "--statement")?;
    let reason = option_value(args, "--reason");
    let actor = stamped(args, "user");
    with_flow_lock(dir, || {
        ensure_flow(dir)?;
        append_event(
            dir,
            &Event::DeviationRecorded {
                id: id.clone(),
                statement,
                reason,
                actor,
                at: now_id(),
            },
        )?;
        rebuild(dir)?;
        println!("deviation recorded: {id}");
        Ok(())
    })
}

// ─── attach ──────────────────────────────────────────────────────────────────

fn cmd_attach(args: &[String]) -> Result<()> {
    if args.len() != 4 || args[2] != "--as" || args[3] != "child" {
        return Err("usage: harness-flow attach <parent-dir> <child-dir> --as child".to_string());
    }
    let parent = Path::new(&args[0]);
    let child = Path::new(&args[1]);
    if !events_path(child).exists() {
        return Err(format!("child flow not found: {}", child.display()));
    }
    let child = fs::canonicalize(child).map_err(|e| e.to_string())?;
    with_flow_lock(parent, || {
        ensure_flow(parent)?;
        append_event(
            parent,
            &Event::ChildAttached {
                path: child.display().to_string(),
            },
        )?;
        rebuild(parent)?;
        Ok(())
    })
}

// ─── locks / paths ───────────────────────────────────────────────────────────

struct FlowLock {
    path: PathBuf,
}

impl Drop for FlowLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn with_flow_lock<T>(dir: &Path, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let _lock = acquire_flow_lock(dir)?;
    f()
}

fn acquire_flow_lock(dir: &Path) -> Result<FlowLock> {
    fs::create_dir_all(progress_dir(dir)).map_err(|e| e.to_string())?;
    let path = progress_dir(dir).join(".lock");
    let start = Instant::now();
    let timeout = lock_timeout();
    loop {
        match fs::create_dir(&path) {
            Ok(()) => {
                let owner = path.join("owner");
                let _ = fs::write(owner, format!("pid={}\n", process::id()));
                return Ok(FlowLock { path });
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                if start.elapsed() >= timeout {
                    return Err(format!(
                        "flow lock is held: {}. Another writer is updating this flow.",
                        path.display()
                    ));
                }
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => return Err(err.to_string()),
        }
    }
}

fn lock_timeout() -> Duration {
    env::var("HARNESS_FLOW_LOCK_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(30))
}

// ─── rebuild / persist ───────────────────────────────────────────────────────

fn rebuild(dir: &Path) -> Result<State> {
    let events = read_events(dir)?;
    let mut state = State::default();
    for event in events {
        apply_event(&mut state, event)?;
    }
    write_state(dir, &state)?;
    Ok(state)
}

fn read_events(dir: &Path) -> Result<Vec<Event>> {
    let path = events_path(dir);
    if !path.exists() {
        return Err(format!("flow not found: {}", dir.display()));
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_event)
        .collect()
}

fn ensure_flow(dir: &Path) -> Result<()> {
    if events_path(dir).exists() {
        Ok(())
    } else {
        Err(format!("flow not found: {}", dir.display()))
    }
}

fn apply_event(state: &mut State, event: Event) -> Result<()> {
    state.seq += 1;
    let event_seq = state.seq;
    match event {
        Event::FlowInitialized { flow_id, .. } => {
            state.flow_id = Some(flow_id);
        }
        Event::GateAdded { id, requires } => {
            state.gates.insert(id, Gate { requires });
        }
        Event::AttemptStarted {
            id,
            gate,
            kind,
            actor,
            executor,
            command,
        } => {
            state.attempts.insert(
                id,
                Attempt {
                    seq: event_seq,
                    gate,
                    kind,
                    actor,
                    executor,
                    command,
                    report: None,
                    finished: false,
                    verdicts: Vec::new(),
                },
            );
        }
        Event::AttemptFinished {
            id,
            report,
            verdicts,
        } => {
            let attempt = state
                .attempts
                .get_mut(&id)
                .ok_or_else(|| format!("unknown attempt in event log: {id}"))?;
            attempt.finished = true;
            attempt.report = report;
            attempt.verdicts = verdicts;
        }
        Event::RunRecorded { check, result } => {
            state.run_results.insert(check, result);
        }
        Event::ArtifactAdded {
            id,
            path,
            kind,
            hash,
            producer,
            deps,
        } => {
            state.artifacts.insert(
                id,
                Artifact {
                    path,
                    kind,
                    producer,
                    hash,
                    deps,
                },
            );
        }
        Event::OverrideRecorded {
            check,
            gate,
            reason,
            actor,
            at,
        } => {
            state.overrides.push(Override {
                check,
                gate,
                reason,
                actor,
                at,
            });
        }
        Event::DecisionRecorded {
            id,
            question,
            choice,
            reason,
            actor,
            at,
        } => {
            state.decisions.push(Decision {
                id,
                question,
                choice,
                reason,
                actor,
                at,
            });
        }
        Event::DeviationRecorded {
            id,
            statement,
            reason,
            actor,
            at,
        } => {
            state.deviations.push(Deviation {
                id,
                statement,
                reason,
                actor,
                at,
            });
        }
        Event::ChildAttached { path } => {
            state.children.push(path);
        }
    }
    Ok(())
}

fn append_event(dir: &Path, event: &Event) -> Result<()> {
    fs::create_dir_all(progress_dir(dir)).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path(dir))
        .map_err(|e| e.to_string())?;
    let line = serde_json::to_string(event).map_err(|e| e.to_string())?;
    writeln!(file, "{line}").map_err(|e| e.to_string())
}

fn parse_event(line: &str) -> Result<Event> {
    serde_json::from_str(line).map_err(|err| format!("invalid event log line: {err}: {line}"))
}

// ─── template / protocol parsing ─────────────────────────────────────────────

fn parse_template_text(text: &str) -> Result<(Option<String>, Vec<GateSpec>)> {
    let template: FlowTemplate = toml::from_str(text).map_err(|e| e.to_string())?;
    let mut seen = BTreeSet::new();
    for gate in &template.gates {
        if gate.id.is_empty() {
            return Err("gate missing id".to_string());
        }
        if !seen.insert(gate.id.clone()) {
            return Err(format!("duplicate gate id: {}", gate.id));
        }
    }
    Ok((template.flow.and_then(|flow| flow.id), template.gates))
}

fn persist_flow_template(dir: &Path, template_text: &str) -> Result<()> {
    let path = dir.join(FLOW_TEMPLATE_FILE);
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    match fs::read_to_string(&path) {
        Ok(existing) if existing == template_text => Ok(()),
        Ok(_) => Err(format!(
            "{} already exists with different content",
            path.display()
        )),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::write(path, template_text).map_err(|e| e.to_string())
        }
        Err(err) => Err(err.to_string()),
    }
}

// Absent file = no contract declared; flow runs no checks and passes attempts trivially.
fn load_protocol(dir: &Path) -> Result<Protocol> {
    let path = dir.join(PROTOCOL_FILE);
    if !path.exists() {
        return Ok(Protocol::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let protocol: Protocol =
        toml::from_str(&text).map_err(|e| format!("protocol.toml parse error: {e}"))?;
    validate_protocol(&protocol)?;
    Ok(protocol)
}

fn validate_protocol(protocol: &Protocol) -> Result<()> {
    let mut seen = BTreeMap::<&str, &str>::new();
    for check in &protocol.checks {
        if check.id.trim().is_empty() {
            return Err("protocol.toml invalid: check id is empty".to_string());
        }
        if check.gate.trim().is_empty() {
            return Err(format!(
                "protocol.toml invalid: check {} has empty gate",
                check.id
            ));
        }
        if let Some(prev_gate) = seen.insert(check.id.as_str(), check.gate.as_str()) {
            return Err(format!(
                "protocol.toml invalid: duplicate check id {} in gates {} and {}",
                check.id, prev_gate, check.gate
            ));
        }
    }
    Ok(())
}

// ─── gate evaluation (derived status) ────────────────────────────────────────

fn evaluate_gate(ctx: &Ctx, gate: &str) -> (String, Vec<CheckResult>) {
    let checks: Vec<&Check> = ctx
        .protocol
        .checks
        .iter()
        .filter(|c| c.gate == gate)
        .collect();
    if checks.is_empty() {
        // No checks declared: passed once any attempt finished, else pending.
        let any_finished = ctx
            .state
            .attempts
            .values()
            .any(|a| a.gate == gate && a.finished);
        let status = if any_finished {
            GATE_PASSED
        } else {
            GATE_PENDING
        };
        return (status.to_string(), vec![]);
    }
    let overridden: BTreeSet<&str> = ctx
        .state
        .overrides
        .iter()
        .map(|o| o.check.as_str())
        .collect();
    let mut results = Vec::new();
    let mut any_fail = false;
    for check in checks {
        if overridden.contains(check.id.as_str()) {
            results.push(CheckResult {
                id: check.id.clone(),
                pass: true,
                detail: "overridden".to_string(),
            });
            continue;
        }
        let r = eval_check(ctx, check);
        if !r.pass {
            any_fail = true;
        }
        results.push(r);
    }
    let status = if any_fail { GATE_FAILED } else { GATE_PASSED };
    (status.to_string(), results)
}

fn eval_check(ctx: &Ctx, check: &Check) -> CheckResult {
    let r = match check.kind {
        CheckKind::Audit => eval_audit(ctx.dir, ctx.state, check),
        CheckKind::Run => eval_run(ctx.state, check),
        CheckKind::Exists => eval_exists(ctx.dir, check),
        CheckKind::Agree => eval_agree(ctx.dir, check),
        CheckKind::Near => eval_near(ctx.dir, check),
        CheckKind::Fresh => eval_fresh(ctx.dir, ctx.state, check),
        CheckKind::Cover => eval_cover(ctx.dir, check),
    };
    CheckResult {
        id: check.id.clone(),
        pass: r.0,
        detail: r.1,
    }
}

// Producer and auditor must differ by identity (env FLOW_ACTOR_ID or ppid:<n>);
// equality means the same process produced and audited. The audit report's
// content is pinned at attempt-finish — post-finish edits invalidate the audit.
fn eval_audit(dir: &Path, state: &State, check: &Check) -> (bool, String) {
    let producers: Vec<&Attempt> = state
        .attempts
        .values()
        .filter(|a| a.gate == check.gate && a.kind != ATTEMPT_AUDIT && a.finished)
        .collect();
    let auditors: Vec<&Attempt> = state
        .attempts
        .values()
        .filter(|a| a.gate == check.gate && a.kind == ATTEMPT_AUDIT && a.finished)
        .collect();
    if producers.is_empty() {
        return (false, "no producer attempt to audit".to_string());
    }
    if auditors.is_empty() {
        return (false, "no audit attempt finished".to_string());
    }
    for v in &auditors {
        let report = match v.report.as_ref() {
            Some(r) => r,
            None => {
                return (
                    false,
                    format!("audit attempt {} has no report", v.actor.label),
                )
            }
        };
        match file_hash(&dir.join(&report.path)) {
            Ok(h) if h == report.hash => {}
            Ok(_) => {
                return (
                    false,
                    format!("audit report mutated since finish: {}", report.path),
                )
            }
            Err(e) => {
                return (
                    false,
                    format!("cannot hash audit report {}: {e}", report.path),
                )
            }
        }
        for p in &producers {
            if p.actor.identity == v.actor.identity {
                return (
                    false,
                    format!(
                        "self-audit: identity {} produced and audited",
                        v.actor.identity
                    ),
                );
            }
        }
    }
    (
        true,
        format!("audited by {} distinct actor(s)", auditors.len()),
    )
}

fn eval_run(state: &State, check: &Check) -> (bool, String) {
    let cmd = check.cmd.as_deref().unwrap_or("<no cmd>");
    match state.run_results.get(&check.id) {
        Some(r) if r.ok => (true, format!("exit 0: {cmd}")),
        Some(r) => (
            false,
            format!(
                "nonzero: {cmd} — {}",
                r.output.lines().last().unwrap_or("").trim()
            ),
        ),
        None => (
            false,
            "not yet evaluated; finish an attempt on the gate".to_string(),
        ),
    }
}

fn run_check_command(dir: &Path, check: &Check) -> RunResult {
    let cmd = match check.cmd.as_deref() {
        Some(c) => c,
        None => {
            return RunResult {
                ok: false,
                output: "run check missing cmd".to_string(),
                at: now_id(),
            }
        }
    };
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(dir)
        .output();
    let (ok, merged) = match output {
        Ok(o) => {
            let merged = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            (o.status.success(), merged)
        }
        Err(e) => (false, format!("spawn failed: {e}")),
    };
    RunResult {
        ok,
        output: merged,
        at: now_id(),
    }
}

fn eval_exists(dir: &Path, check: &Check) -> (bool, String) {
    let mut missing = Vec::new();
    for path in &check.paths {
        if !dir.join(path).exists() {
            missing.push(path.clone());
        }
    }
    for field in &check.fields {
        let (file, key) = match field.split_once(':') {
            Some((f, k)) => (f, k),
            None => {
                missing.push(format!("{field} (malformed; expected path:field)"));
                continue;
            }
        };
        let json = match read_json(&dir.join(file)) {
            Some(v) => v,
            None => {
                missing.push(format!("{file}:{key} (file missing or not json)"));
                continue;
            }
        };
        if pick(&json, key).is_none() {
            missing.push(format!("{file}:{key}"));
        }
    }
    if missing.is_empty() {
        let n = check.paths.len() + check.fields.len();
        (true, format!("{n} declared paths/fields present"))
    } else {
        (false, format!("missing: {}", missing.join(", ")))
    }
}

fn eval_cover(dir: &Path, check: &Check) -> (bool, String) {
    let pattern = match check.pattern.as_deref() {
        Some(g) if !g.trim().is_empty() => g,
        _ => return (false, "cover check needs `pattern`".to_string()),
    };
    if check.paths.is_empty() {
        return (false, "cover check needs `paths`".to_string());
    }

    let expected: BTreeSet<String> = check.paths.iter().cloned().collect();
    let observed = match relative_files_matching(dir, pattern) {
        Ok(paths) => paths,
        Err(e) => return (false, e),
    };

    let missing: Vec<String> = expected.difference(&observed).cloned().collect();
    let extra: Vec<String> = observed.difference(&expected).cloned().collect();
    if missing.is_empty() && extra.is_empty() {
        (true, format!("{} path(s) covered", expected.len()))
    } else {
        let mut parts = Vec::new();
        if !missing.is_empty() {
            parts.push(format!("missing: {}", missing.join(", ")));
        }
        if !extra.is_empty() {
            parts.push(format!("extra: {}", extra.join(", ")));
        }
        (false, parts.join("; "))
    }
}

fn eval_agree(dir: &Path, check: &Check) -> (bool, String) {
    if check.against.len() < 2 {
        return (
            false,
            "agree check needs at least two `against` entries (path:field each)".to_string(),
        );
    }
    let mut values: Vec<(String, serde_json::Value)> = Vec::new();
    for entry in &check.against {
        let (file, key) = match entry.split_once(':') {
            Some(pair) => pair,
            None => return (false, format!("malformed against entry: {entry}")),
        };
        let json = match read_json(&dir.join(file)) {
            Some(v) => v,
            None => return (false, format!("cannot read {file}")),
        };
        let value = match pick(&json, key) {
            Some(v) => v,
            None => return (false, format!("missing {file}:{key}")),
        };
        values.push((entry.clone(), value));
    }
    let head = &values[0].1;
    for (label, value) in values.iter().skip(1) {
        if value != head {
            return (false, format!("{} differs from {}", label, values[0].0));
        }
    }
    (true, format!("{} entries agree", values.len()))
}

fn eval_near(dir: &Path, check: &Check) -> (bool, String) {
    if check.compare.is_empty() {
        return (
            false,
            "near check needs [[checks.compare]] entries".to_string(),
        );
    }
    let mut details = Vec::new();
    let mut any_fail = false;
    for c in &check.compare {
        let a_path = dir.join(&c.actual.path);
        let r_path = dir.join(&c.reference.path);
        let a = match read_json(&a_path).and_then(|v| pick_number(&v, &c.actual.field)) {
            Some(v) => v,
            None => {
                any_fail = true;
                details.push(format!(
                    "missing actual {}:{}",
                    c.actual.path, c.actual.field
                ));
                continue;
            }
        };
        let r = match read_json(&r_path).and_then(|v| pick_number(&v, &c.reference.field)) {
            Some(v) => v,
            None => {
                any_fail = true;
                details.push(format!(
                    "missing reference {}:{}",
                    c.reference.path, c.reference.field
                ));
                continue;
            }
        };
        let diff = (a - r).abs();
        let pass_abs = c.tolerance.abs.map(|t| diff <= t).unwrap_or(false);
        let pass_rel = c
            .tolerance
            .rel
            .map(|t| diff / r.abs().max(1e-30) <= t)
            .unwrap_or(false);
        let pass_sigma = c
            .tolerance
            .sigma
            .map(|t| {
                c.uncertainty
                    .as_ref()
                    .and_then(|u| {
                        read_json(&dir.join(&u.path)).and_then(|v| pick_number(&v, &u.field))
                    })
                    .map(|s| diff / s.abs().max(1e-30) <= t)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let ok = pass_abs || pass_rel || pass_sigma;
        if !ok {
            any_fail = true;
        }
        details.push(format!("|{a}-{r}|={diff:.3e}"));
    }
    (!any_fail, details.join("; "))
}

fn eval_fresh(dir: &Path, state: &State, check: &Check) -> (bool, String) {
    if check.paths.is_empty() || check.against.is_empty() {
        return (
            false,
            "fresh check needs `paths` (artifacts) and `against` (sources)".to_string(),
        );
    }
    for path in &check.paths {
        let artifact = match state.artifacts.values().find(|a| a.path == *path) {
            Some(a) => a,
            None => return (false, format!("no artifact registered for {path}")),
        };
        match file_hash(&dir.join(path)) {
            Ok(h) if h == artifact.hash => {}
            Ok(_) => {
                return (
                    false,
                    format!("artifact mutated since registration: {path}"),
                )
            }
            Err(e) => return (false, format!("cannot hash artifact {path}: {e}")),
        }
        for source in &check.against {
            let registered = match artifact.deps.get(source) {
                Some(h) => h,
                None => {
                    return (
                        false,
                        format!("no snapshot of {source} for {path}; re-register the artifact"),
                    )
                }
            };
            match file_hash(&dir.join(source)) {
                Ok(h) if h == *registered => {}
                Ok(_) => {
                    return (
                        false,
                        format!("source changed since registration: {source}"),
                    )
                }
                Err(e) => return (false, format!("cannot hash source {source}: {e}")),
            }
        }
    }
    (
        true,
        format!("{} artifact(s) and sources unchanged", check.paths.len()),
    )
}

fn snapshot_deps(dir: &Path, artifact_path: &str) -> Result<BTreeMap<String, String>> {
    let protocol = load_protocol(dir)?;
    let mut deps = BTreeMap::new();
    for check in &protocol.checks {
        if check.kind != CheckKind::Fresh {
            continue;
        }
        if !check.paths.iter().any(|p| p == artifact_path) {
            continue;
        }
        for source in &check.against {
            if deps.contains_key(source) {
                continue;
            }
            let resolved = dir.join(source);
            if !resolved.exists() {
                return Err(format!(
                    "fresh-check source missing at registration of {artifact_path}: {source}"
                ));
            }
            deps.insert(source.clone(), file_hash(&resolved)?);
        }
    }
    Ok(deps)
}

// ─── derived queries ─────────────────────────────────────────────────────────

fn gate_status(ctx: &Ctx, gate: &str) -> String {
    if !ctx.state.gates.contains_key(gate) {
        return "missing".to_string();
    }
    evaluate_gate(ctx, gate).0
}

fn gate_passed(ctx: &Ctx, gate: &str) -> bool {
    gate_status(ctx, gate) == GATE_PASSED && requirements_passed(ctx, gate)
}

fn requirements_passed(ctx: &Ctx, gate: &str) -> bool {
    let Some(spec) = ctx.state.gates.get(gate) else {
        return false;
    };
    spec.requires
        .iter()
        .all(|required| gate_status(ctx, required) == GATE_PASSED)
}

fn print_deviation(pad: &str, id: &str, statement: &str, reason: Option<&str>) {
    let trimmed = reason.map(str::trim).filter(|r| !r.is_empty());
    if let Some(r) = trimmed {
        println!("{pad}  ⚠ {id} — {statement} ({r})");
    } else {
        println!("{pad}  ⚠ {id} — {statement}");
    }
}

fn ready_gates(ctx: &Ctx) -> Vec<String> {
    ctx.state
        .gates
        .keys()
        .filter(|gate| gate_status(ctx, gate) != GATE_PASSED && requirements_passed(ctx, gate))
        .cloned()
        .collect()
}

fn dag_order(state: &State) -> Vec<&str> {
    let mut order: Vec<&str> = Vec::new();
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    for id in state.gates.keys() {
        dag_visit(state, id.as_str(), &mut visited, &mut order);
    }
    order
}

fn dag_visit<'a>(
    state: &'a State,
    id: &'a str,
    visited: &mut BTreeSet<&'a str>,
    order: &mut Vec<&'a str>,
) {
    if !visited.insert(id) {
        return;
    }
    if let Some(g) = state.gates.get(id) {
        for req in &g.requires {
            dag_visit(state, req.as_str(), visited, order);
        }
    }
    order.push(id);
}

// Latest audit attempt that voted on this claim wins. Returns None when no
// audit subagent has issued a verdict yet — render.py paints "unknown" chips.
fn claim_verdict<'a>(state: &'a State, claim_id: &str) -> Option<&'a Verdict> {
    state
        .attempts
        .values()
        .filter(|a| a.finished && a.kind == ATTEMPT_AUDIT)
        .filter_map(|a| {
            a.verdicts
                .iter()
                .find(|v| v.claim == claim_id)
                .map(|v| (a.seq, v))
        })
        .max_by_key(|(seq, _)| *seq)
        .map(|(_, v)| v)
}

// ─── option parsing ──────────────────────────────────────────────────────────

fn option_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn required_option(args: &[String], flag: &str) -> Result<String> {
    option_value(args, flag).ok_or_else(|| format!("missing {flag}"))
}

// ─── json helpers ────────────────────────────────────────────────────────────

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn pick(value: &serde_json::Value, field: &str) -> Option<serde_json::Value> {
    let mut cur = value;
    for part in field.split('.') {
        cur = cur.get(part)?;
    }
    Some(cur.clone())
}

fn pick_number(value: &serde_json::Value, field: &str) -> Option<f64> {
    pick(value, field)?.as_f64()
}

fn relative_files_matching(root: &Path, pattern: &str) -> Result<BTreeSet<String>> {
    let prefix = pattern_scan_prefix(pattern);
    let start = if prefix.is_empty() {
        root.to_path_buf()
    } else {
        root.join(prefix)
    };
    let metadata = match fs::symlink_metadata(&start) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => return Err(e.to_string()),
    };
    let mut out = BTreeSet::new();
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Ok(out);
    }
    if file_type.is_file() {
        let rel = relative_path(root, &start)?;
        if wildcard_match(pattern, &rel) {
            out.insert(rel);
        }
        return Ok(out);
    }
    if file_type.is_dir() {
        collect_relative_files_matching(root, &start, pattern, &mut out)?;
    }
    Ok(out)
}

fn collect_relative_files_matching(
    root: &Path,
    dir: &Path,
    pattern: &str,
    out: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_relative_files_matching(root, &path, pattern, out)?;
        } else if file_type.is_file() {
            let rel = relative_path(root, &path)?;
            if wildcard_match(pattern, &rel) {
                out.insert(rel);
            }
        }
    }
    Ok(())
}

fn pattern_scan_prefix(pattern: &str) -> &str {
    let wildcard_at = pattern.find('*').unwrap_or(pattern.len());
    let literal = &pattern[..wildcard_at];
    literal
        .rfind('/')
        .map(|i| &literal[..=i])
        .unwrap_or_default()
}

fn relative_path(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(|e| e.to_string())?
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == value;
    }
    let anchored_start = !pattern.starts_with('*');
    let anchored_end = !pattern.ends_with('*');
    let mut rest = value;
    for (i, part) in pattern.split('*').filter(|p| !p.is_empty()).enumerate() {
        let Some(pos) = rest.find(part) else {
            return false;
        };
        if i == 0 && anchored_start && pos != 0 {
            return false;
        }
        rest = &rest[pos + part.len()..];
    }
    !anchored_end
        || pattern
            .rsplit('*')
            .next()
            .map(|p| value.ends_with(p))
            .unwrap_or(true)
}

// ─── state.toml projection ───────────────────────────────────────────────────

fn write_state(dir: &Path, state: &State) -> Result<()> {
    fs::create_dir_all(progress_dir(dir)).map_err(|e| e.to_string())?;
    let out = toml::to_string_pretty(&StateFile::from(state)).map_err(|e| e.to_string())?;
    fs::write(state_path(dir), out).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct StateFile<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    flow: Option<StateFlow<'a>>,
    gates: &'a BTreeMap<String, Gate>,
    attempts: &'a BTreeMap<String, Attempt>,
    artifacts: &'a BTreeMap<String, Artifact>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    run_results: &'a BTreeMap<String, RunResult>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    overrides: &'a [Override],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    decisions: &'a [Decision],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    deviations: &'a [Deviation],
    children: StateChildren<'a>,
}

#[derive(Serialize)]
struct StateFlow<'a> {
    id: &'a str,
}

#[derive(Serialize)]
struct StateChildren<'a> {
    paths: &'a [String],
}

impl<'a> From<&'a State> for StateFile<'a> {
    fn from(state: &'a State) -> Self {
        Self {
            flow: state.flow_id.as_deref().map(|id| StateFlow { id }),
            gates: &state.gates,
            attempts: &state.attempts,
            artifacts: &state.artifacts,
            run_results: &state.run_results,
            overrides: &state.overrides,
            decisions: &state.decisions,
            deviations: &state.deviations,
            children: StateChildren {
                paths: &state.children,
            },
        }
    }
}

// ─── misc ────────────────────────────────────────────────────────────────────

fn file_hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn now_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{nanos}", process::id())
}

fn flow_id_from_path(dir: &Path) -> String {
    dir.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("flow")
        .to_string()
}

fn progress_dir(dir: &Path) -> PathBuf {
    dir.join("progress")
}

fn events_path(dir: &Path) -> PathBuf {
    progress_dir(dir).join("events.jsonl")
}

fn state_path(dir: &Path) -> PathBuf {
    progress_dir(dir).join("state.toml")
}

#[cfg(test)]
mod tests {
    use super::file_hash;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn file_hash_matches_known_vector() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("harness-flow-hash-{}-{nanos}", std::process::id()));
        fs::write(&path, b"abc").unwrap();
        assert_eq!(
            file_hash(&path).unwrap(),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        fs::remove_file(path).unwrap();
    }
}
