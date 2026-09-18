use super::*;
use crate::fixture::TempRepo;

const SPEC_TEXT: &str = "# Spec\n\n## Capability Registry\n\n\
    | # | Capability | Status | Implementation |\n| --- | --- | --- | --- |\n\
    | 1 | `alpha` | shipped | [Current](../impl/current/alpha.md) |\n\
    | 2 | `beta` | planned | [Open work](../impl/plan.md) |\n\n## After\n";

const WORKFLOW_TEXT: &str = "# Workflow\n\n| Capability id | Group abbrev |\n| --- | --- |\n\
    | `alpha` | `al` |\n| `beta` | `be` |\n";

const RECORD: &str = "docs/impl/records/0001-al-first-task.md";

fn agent_task(id: &str, deps: &str) -> String {
    format!(
        "#### {id}\n\nDo it.\n\n\
         - Serves: `beta` -- [Spec](../design/spec.md)\n- Agent status: CLEAR\n\
         - Dependencies: {deps}\n- User-visible outcome: o\n- Scope boundary: s\n\
         - Data and artifact paths: p\n- Execution path:\n  multi-line\n  execution\n\
         - Acceptance gates: g\n- Documentation target: d\n- Review checkpoint: none.\n\n"
    )
}

fn valid_plan() -> String {
    format!(
        "# Plan\n\n## Agent Implementation Tasks\n\n### Beta -- `beta`\n\n{}{}\
         ## Human-Assisted Tasks\n\n### Beta -- `beta`\n\n#### wait-human\n\nDecide.\n\n\
         - Serves: `beta` -- x\n- Human status: HUMAN-GATED\n- Dependencies: none.\n\
         - Requested input or decision: d\n- Unblocks: `finish-beta`.\n",
        agent_task("build-beta", "[First](records/0001-al-first-task.md)."),
        agent_task("finish-beta", "`build-beta`; `wait-human`."),
    )
}

fn repo(plan: &str) -> TempRepo {
    TempRepo::with(&[
        (SPEC, SPEC_TEXT),
        (WORKFLOW, WORKFLOW_TEXT),
        (PLAN, plan),
        ("docs/impl/records/README.md", "| [0001](0001-al-first-task.md) | x |\n"),
        ("docs/impl/records/template.md", "# Template\n"),
        (RECORD, "# First\n\n- Id / capability / checkpoint: `first-task` / `alpha` / none\n"),
    ])
}

fn lint_repo(repo: &TempRepo) -> Vec<String> {
    let (inputs, mut problems) = load(repo.root()).expect("load the planning documents");
    problems.extend(lint(&inputs));
    problems
}

#[test]
fn lint_accepts_consistent_documents() {
    let problems = lint_repo(&repo(&valid_plan()));
    assert!(problems.is_empty(), "unexpected problems:\n{}", problems.join("\n"));
}

#[test]
fn parse_plan_joins_multiline_fields() {
    let plan = parse_plan(&valid_plan());
    assert_eq!(plan.tasks[0].field("Execution path"), "multi-line execution");
    assert_eq!(plan.tasks.len(), 3);
    assert_eq!(plan.groups.len(), 2);
}

#[test]
fn lint_detects_defects() {
    let replaced = |old: &str, new: &str| repo(&valid_plan().replacen(old, new, 1));
    let with_file = |path: &str, text: &str| {
        let repo = repo(&valid_plan());
        repo.write(path, text);
        repo
    };
    let cases = [
        (
            "unknown dependency",
            replaced("`build-beta`;", "`ghost-task`;"),
            "dependency `ghost-task` is not an open task",
        ),
        (
            "missing record",
            replaced("0001-al-first-task", "0009-al-gone"),
            "0009-al-gone.md does not exist",
        ),
        (
            "cycle",
            replaced("[First](records/0001-al-first-task.md).", "`finish-beta`."),
            "dependency cycle",
        ),
        (
            "missing field",
            replaced("- Scope boundary: s\n", ""),
            r#"missing field "Scope boundary""#,
        ),
        (
            "bad status",
            replaced("Agent status: CLEAR", "Agent status: DONE"),
            r#"Agent status "DONE""#,
        ),
        (
            "wrong serves",
            replaced("- Serves: `beta`", "- Serves: `alpha`"),
            "Serves must start with `beta`",
        ),
        (
            "group not in registry",
            replaced("### Beta -- `beta`", "### Gamma -- `gamma`"),
            r#"group "gamma" is not in the registry"#,
        ),
        (
            "accepted task still planned",
            replaced("#### build-beta", "#### first-task"),
            r#"task "first-task" is still in the plan"#,
        ),
        (
            "duplicate task",
            replaced("#### finish-beta", "#### build-beta"),
            r#"duplicate task id "build-beta""#,
        ),
        (
            "planned capability without tasks",
            repo("# Plan\n\n## Agent Implementation Tasks\n"),
            r#"planned capability "beta" has no open tasks"#,
        ),
        (
            "shipped capability without current page",
            with_file(SPEC, &SPEC_TEXT.replacen("../impl/current/alpha.md", "../impl/plan.md", 1)),
            r#"shipped capability "alpha" must link its current-state page"#,
        ),
        (
            "record not indexed",
            with_file("docs/impl/records/README.md", "empty\n"),
            "not linked from the records README",
        ),
        (
            "record with unknown group",
            with_file("docs/impl/records/0002-zz-other.md", "x"),
            "unknown group",
        ),
        (
            "malformed record name",
            with_file("docs/impl/records/notes.md", "x"),
            "name must be NNNN-<group>-<task-id>.md",
        ),
        (
            "reused record sequence",
            with_file(
                "docs/impl/records/0001-be-other-task.md",
                "- Id / capability / checkpoint: `other-task` / `beta` / none\n",
            ),
            "sequence 0001 is already used",
        ),
        ("record without id line", with_file(RECORD, "# First\n"), "first scope line must be"),
    ];
    for (name, repo, want) in cases {
        let problems = lint_repo(&repo).join("\n");
        assert!(
            problems.contains(want),
            "{name}: want a problem containing {want:?}, got:\n{problems}"
        );
    }
}

#[test]
fn load_fails_without_the_plan() {
    let repo = repo(&valid_plan());
    repo.remove(PLAN);
    let err = load(repo.root()).expect_err("the plan is missing");
    assert!(err.to_string().contains(PLAN), "{err}");
}

#[test]
fn status_reports_eligible_work() {
    let plan = parse_plan(&valid_plan());
    let status = Status::new(&plan);
    assert_eq!((status.agent_tasks, status.human_tasks), (2, 1));
    let ids = |tasks: &[&Task]| tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(&status.eligible), ["build-beta"]);
    assert_eq!(ids(&status.waiting_human), ["wait-human"]);
    assert_eq!(
        status.to_string(),
        "open tasks: 3 (2 agent, 1 human)\n\
         next agent task: build-beta (beta, CLEAR)\n\
         human action available: wait-human (HUMAN-GATED)\n"
    );
}

#[test]
fn record_names_split_by_the_longest_group() {
    let groups: BTreeMap<String, String> =
        [("a", "dist"), ("b", "dist-extra")].map(|(c, g)| (c.to_owned(), g.to_owned())).into();
    let record = parse_record_name("0007-dist-extra-ship-it.md", &groups).expect("valid name");
    assert_eq!((record.sequence.as_str(), record.task_id.as_str()), ("0007", "ship-it"));
    assert!(parse_record_name("0007-dist-.md", &groups).is_err());
    assert!(parse_record_name("007-dist-x.md", &groups).is_err());
}
