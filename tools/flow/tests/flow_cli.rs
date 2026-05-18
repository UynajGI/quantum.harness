use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> String {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_harness-flow") {
        return path;
    }
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("harness-flow");
    path.to_string_lossy().to_string()
}

fn tmp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "harness-flow-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin()).args(args).output().unwrap()
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(bin());
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().unwrap()
}

fn assert_ok(args: &[&str]) -> String {
    let output = run(args);
    assert!(
        output.status.success(),
        "command failed\nargs: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn assert_fail(args: &[&str]) -> String {
    let output = run(args);
    assert!(
        !output.status.success(),
        "command unexpectedly passed\nargs: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn assert_fail_with_env(args: &[&str], envs: &[(&str, &str)]) -> String {
    let output = run_with_env(args, envs);
    assert!(
        !output.status.success(),
        "command unexpectedly passed\nargs: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn init_from_template_and_require_gate() {
    let root = tmp_dir("init");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(
        &template,
        r#"
[flow]
id = "idea_to_verified_plan"

[[gates]]
id = "ideas"

[[gates]]
id = "critic"
requires = ["ideas"]
invalidates = ["revision", "verify"]

[[gates]]
id = "revision"
requires = ["critic"]

[[gates]]
id = "verify"
requires = ["revision"]
"#,
    );

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);

    let status = assert_ok(&["status", run_dir.to_str().unwrap()]);
    assert!(status.contains("ideas"));
    assert!(status.contains("pending"));

    let err = assert_fail(&["require", run_dir.to_str().unwrap(), "ideas"]);
    assert!(err.contains("not passed"));
}

#[test]
fn init_copies_external_template_into_run_dir() {
    let root = tmp_dir("init-template-copy");
    let template = root.join("templates").join("custom.toml");
    let run_dir = root.join("run");
    let template_text = r#"
[flow]
id = "copied_template"

[[gates]]
id = "source"

[[gates]]
id = "close"
requires = ["source"]
"#;
    write(&template, template_text);

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);

    let copied = fs::read_to_string(run_dir.join("flow.toml")).unwrap();
    assert_eq!(copied, template_text);
    let status = assert_ok(&["status", run_dir.to_str().unwrap()]);
    assert!(status.contains("copied_template"));
    assert!(status.lines().any(|line| line.starts_with("source\t")));
}

#[test]
fn attempt_finish_passes_with_no_declared_checks() {
    // No protocol.toml = no checks declared = trivial pass.
    let root = tmp_dir("attempt");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(
        &template,
        r#"
[[gates]]
id = "ideas"

[[gates]]
id = "critic"
requires = ["ideas"]
"#,
    );

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    let attempt = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "ideas",
        "--kind",
        "produce",
        "--actor",
        "agent:main",
    ]);
    let attempt = attempt.trim();
    assert!(attempt.starts_with('a'));

    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        attempt,
    ]);

    assert_ok(&["require", run_dir.to_str().unwrap(), "ideas"]);
    let next = assert_ok(&["next", run_dir.to_str().unwrap()]);
    assert!(next.lines().any(|line| line.trim() == "critic"));
}

#[test]
fn artifact_hash_change_invalidates_downstream_gates() {
    let root = tmp_dir("invalidate");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    let protocol = run_dir.join("protocol_doc.toml");
    write(
        &template,
        r#"
[[gates]]
id = "protocol"
invalidates = ["plan", "script"]

[[gates]]
id = "plan"
requires = ["protocol"]

[[gates]]
id = "script"
requires = ["plan"]
"#,
    );
    write(&protocol, "claim = 1\n");

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    let attempt = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "protocol",
        "--kind",
        "produce",
        "--actor",
        "agent:author",
    ]);
    assert_ok(&[
        "artifact",
        "add",
        run_dir.to_str().unwrap(),
        "protocol",
        protocol.to_str().unwrap(),
        "--kind",
        "protocol",
        "--producer",
        attempt.trim(),
    ]);
    let state = fs::read_to_string(run_dir.join("progress").join("state.toml")).unwrap();
    assert!(state.contains("sha256:"));
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        attempt.trim(),
    ]);

    let plan_attempt = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "plan",
        "--kind",
        "produce",
        "--actor",
        "agent:planner",
    ]);
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        plan_attempt.trim(),
    ]);

    write(&protocol, "claim = 2\n");
    let repair = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "protocol",
        "--kind",
        "produce",
        "--actor",
        "agent:author",
    ]);
    assert_ok(&[
        "artifact",
        "add",
        run_dir.to_str().unwrap(),
        "protocol",
        protocol.to_str().unwrap(),
        "--kind",
        "protocol",
        "--producer",
        repair.trim(),
    ]);
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        repair.trim(),
    ]);

    let status = assert_ok(&["status", run_dir.to_str().unwrap()]);
    assert!(status.lines().any(|line| line.starts_with("protocol\tpassed")));
    assert!(status.lines().any(|line| line.starts_with("plan\tinvalidated")));
    assert!(status.lines().any(|line| line.starts_with("script\tinvalidated")));
}

#[test]
fn held_lock_blocks_second_writer() {
    let root = tmp_dir("lock");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(&template, "[[gates]]\nid = \"source\"\n");

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    fs::create_dir(run_dir.join("progress").join(".lock")).unwrap();

    let err = assert_fail_with_env(
        &["status", run_dir.to_str().unwrap()],
        &[("HARNESS_FLOW_LOCK_TIMEOUT_MS", "10")],
    );
    assert!(err.contains("flow lock is held"));
}

#[test]
fn stale_attempt_cannot_finish_after_newer_one_finished() {
    let root = tmp_dir("stale-attempt");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(&template, "[[gates]]\nid = \"verify\"\n");

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    let old = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "verify",
        "--kind",
        "audit",
        "--actor",
        "agent:slow-reviewer",
    ]);
    let new = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "verify",
        "--kind",
        "audit",
        "--actor",
        "agent:current-reviewer",
    ]);

    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        new.trim(),
    ]);
    let err = assert_fail(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        old.trim(),
    ]);
    assert!(err.contains("stale attempt"));
    assert_ok(&["require", run_dir.to_str().unwrap(), "verify"]);
}

#[test]
fn artifact_invalidation_uses_producer_gate_and_dependency_closure() {
    let root = tmp_dir("artifact-source");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    let source = run_dir.join("sources").join("paper.md");
    write(
        &template,
        r#"
[[gates]]
id = "source"

[[gates]]
id = "plan"
requires = ["source"]

[[gates]]
id = "script"
requires = ["plan"]
"#,
    );
    write(&source, "paper passage\n");

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    let attempt = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "source",
        "--kind",
        "produce",
        "--actor",
        "agent:source-author",
    ]);
    assert_ok(&[
        "artifact",
        "add",
        run_dir.to_str().unwrap(),
        "paper_source",
        source.to_str().unwrap(),
        "--kind",
        "primary",
        "--producer",
        attempt.trim(),
    ]);
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        attempt.trim(),
    ]);

    assert_ok(&[
        "invalidate",
        run_dir.to_str().unwrap(),
        "--from",
        "paper_source",
    ]);

    let status = assert_ok(&["status", run_dir.to_str().unwrap()]);
    assert!(status.lines().any(|line| line.starts_with("source\tinvalidated")));
    assert!(status.lines().any(|line| line.starts_with("plan\tinvalidated")));
    assert!(status.lines().any(|line| line.starts_with("script\tinvalidated")));
}

#[test]
fn parent_flow_tracks_child_flows_recursively() {
    let root = tmp_dir("campaign");
    let template = root.join("template.toml");
    let parent = root.join("campaign");
    let child = parent.join("runs").join("paper-a");
    write(&template, "[[gates]]\nid = \"closed\"\n");

    assert_ok(&[
        "init",
        parent.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    assert_ok(&[
        "init",
        child.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    assert_ok(&[
        "attach",
        parent.to_str().unwrap(),
        child.to_str().unwrap(),
        "--as",
        "child",
    ]);

    let status = assert_ok(&["status", parent.to_str().unwrap(), "--recursive"]);
    assert!(status.contains("children"));
    assert!(status.contains("paper-a"));
}

#[test]
fn check_passes_when_protocol_declares_no_checks_for_gate() {
    let root = tmp_dir("check-empty");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(&template, "[[gates]]\nid = \"source\"\n");

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);
    let stdout = assert_ok(&["check", run_dir.to_str().unwrap(), "source"]);
    assert!(stdout.lines().any(|line| line == "status\tpassed"));
}

#[test]
fn check_runs_declared_run_kind_and_fails_on_nonzero_exit() {
    let root = tmp_dir("check-run");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(&template, "[[gates]]\nid = \"source\"\n");
    fs::create_dir_all(&run_dir).unwrap();
    write(
        &run_dir.join("protocol.toml"),
        r#"
[[checks]]
id = "always_fail"
kind = "run"
gate = "source"
cmd = "exit 1"
"#,
    );

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);

    let err = assert_fail(&["check", run_dir.to_str().unwrap(), "source"]);
    assert!(err.is_empty() || err.contains("status"), "stderr: {err}");
}

#[test]
fn override_records_event_and_satisfies_failing_check() {
    let root = tmp_dir("override");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(&template, "[[gates]]\nid = \"source\"\n");
    fs::create_dir_all(&run_dir).unwrap();
    write(
        &run_dir.join("protocol.toml"),
        r#"
[[checks]]
id = "always_fail"
kind = "run"
gate = "source"
cmd = "exit 1"
"#,
    );

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);

    // Before override, check fails.
    let pre = run(&["check", run_dir.to_str().unwrap(), "source"]);
    assert!(!pre.status.success());

    assert_ok(&[
        "override",
        run_dir.to_str().unwrap(),
        "always_fail",
        "--reason",
        "draft for Slack today",
    ]);

    // After override, check passes.
    let stdout = assert_ok(&["check", run_dir.to_str().unwrap(), "source"]);
    assert!(stdout.lines().any(|line| line == "status\tpassed"));
    assert!(stdout.lines().any(|line| line.contains("overridden")));

    // State.toml shows the override.
    let state = fs::read_to_string(run_dir.join("progress").join("state.toml")).unwrap();
    assert!(state.contains("always_fail"));
    assert!(state.contains("draft for Slack today"));
}

#[test]
fn audit_check_rejects_self_verification() {
    let root = tmp_dir("audit-self");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(&template, "[[gates]]\nid = \"protocol\"\n");
    fs::create_dir_all(&run_dir).unwrap();
    write(
        &run_dir.join("protocol.toml"),
        r#"
[[checks]]
id = "protocol_audit"
kind = "audit"
gate = "protocol"
"#,
    );

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);

    // Author produces; same actor "verifies" → audit fails.
    let producer = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "protocol",
        "--kind",
        "produce",
        "--actor",
        "agent:author",
    ]);
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        producer.trim(),
    ]);

    let auditor = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "protocol",
        "--kind",
        "audit",
        "--actor",
        "agent:author", // SAME actor — should fail audit
    ]);
    let report = run_dir.join("verify").join("self.md");
    write(&report, "self review\n");
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        auditor.trim(),
        "--report",
        report.to_str().unwrap(),
    ]);

    let status = assert_ok(&["status", run_dir.to_str().unwrap()]);
    assert!(status.lines().any(|line| line.starts_with("protocol\tfailed")));
}

#[test]
fn audit_check_passes_with_distinct_actor() {
    let root = tmp_dir("audit-distinct");
    let template = root.join("template.toml");
    let run_dir = root.join("run");
    write(&template, "[[gates]]\nid = \"protocol\"\n");
    fs::create_dir_all(&run_dir).unwrap();
    write(
        &run_dir.join("protocol.toml"),
        r#"
[[checks]]
id = "protocol_audit"
kind = "audit"
gate = "protocol"
"#,
    );

    assert_ok(&[
        "init",
        run_dir.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
    ]);

    let producer = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "protocol",
        "--kind",
        "produce",
        "--actor",
        "agent:author",
    ]);
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        producer.trim(),
    ]);

    let auditor = assert_ok(&[
        "attempt",
        "start",
        run_dir.to_str().unwrap(),
        "protocol",
        "--kind",
        "audit",
        "--actor",
        "agent:independent-reviewer",
    ]);
    let report = run_dir.join("verify").join("independent.md");
    write(&report, "independent review\n");
    assert_ok(&[
        "attempt",
        "finish",
        run_dir.to_str().unwrap(),
        auditor.trim(),
        "--report",
        report.to_str().unwrap(),
    ]);

    assert_ok(&["require", run_dir.to_str().unwrap(), "protocol"]);
}
