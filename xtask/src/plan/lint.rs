//! Integrity checks across the capability registry, the plan and the task records.

use std::collections::{HashMap, HashSet};

use super::{Inputs, Lane, Task};
use crate::markdown::{backticked_ids, is_id};

/// Returns every disagreement between the planning documents; empty when they agree.
pub fn lint(inputs: &Inputs) -> Vec<String> {
    let mut problems = Vec::new();
    let order = lint_registry(inputs, &mut problems);
    lint_group_order(inputs, &order, &mut problems);
    let open = open_tasks(inputs, &mut problems);
    let records = lint_records(inputs, &open, &mut problems);
    for task in &inputs.plan.tasks {
        problems.extend(lint_task(task, &open, &records));
    }
    lint_open_work(inputs, &mut problems);
    if let Some(cycle) = find_cycle(&inputs.plan.tasks) {
        problems.push(format!("plan: dependency cycle {}", cycle.join(" -> ")));
    }
    problems
}

/// Checks registry rows; returns each capability's position in the implementation line.
fn lint_registry<'a>(inputs: &'a Inputs, problems: &mut Vec<String>) -> HashMap<&'a str, usize> {
    let mut order = HashMap::new();
    for (i, capability) in inputs.registry.iter().enumerate() {
        let (id, status) = (&capability.id, &capability.status);
        if order.insert(id.as_str(), i).is_some() {
            problems.push(format!("registry: capability {id:?} listed twice"));
        }
        if status != "planned" && status != "shipped" {
            problems.push(format!(
                "registry: capability {id:?} has status {status:?}, want planned or shipped"
            ));
        }
        if status == "shipped" && !capability.implementation.contains("impl/current/") {
            problems.push(format!(
                "registry: shipped capability {id:?} must link its current-state page"
            ));
        }
        if !inputs.groups.contains_key(id) {
            problems.push(format!(
                "registry: capability {id:?} has no record group in the planning workflow table"
            ));
        }
    }
    if inputs.registry.is_empty() {
        problems.push("registry: no capabilities parsed from the specification".to_owned());
    }
    order
}

/// Plan groups must name registered capabilities, in registry order within each lane.
fn lint_group_order(inputs: &Inputs, order: &HashMap<&str, usize>, problems: &mut Vec<String>) {
    let mut last_in_lane: HashMap<Lane, usize> = HashMap::new();
    for group in &inputs.plan.groups {
        let (at, id) = (group.line, &group.capability);
        let Some(&index) = order.get(id.as_str()) else {
            problems.push(format!("plan:{at}: group {id:?} is not in the registry"));
            continue;
        };
        if let Some(&previous) = last_in_lane.get(&group.lane)
            && index <= previous
        {
            let lane = group.lane;
            problems.push(format!(
                "plan:{at}: group {id:?} is out of registry order in the {lane} lane"
            ));
        }
        last_in_lane.insert(group.lane, index);
    }
}

/// Indexes open tasks by id and reports duplicate ids.
fn open_tasks<'a>(inputs: &'a Inputs, problems: &mut Vec<String>) -> HashMap<&'a str, &'a Task> {
    let mut open = HashMap::new();
    for task in &inputs.plan.tasks {
        if open.insert(task.id.as_str(), task).is_some() {
            problems.push(format!("plan:{}: duplicate task id {:?}", task.line, task.id));
        }
    }
    open
}

/// Checks record names, sequences, id lines and indexing; returns record file names without `.md`.
fn lint_records<'a>(
    inputs: &'a Inputs,
    open: &HashMap<&str, &Task>,
    problems: &mut Vec<String>,
) -> HashSet<&'a str> {
    let mut records = HashSet::new();
    let mut sequences = HashMap::new();
    for record in &inputs.records {
        let (file, task_id) = (&record.file, &record.task_id);
        records.insert(file.trim_end_matches(".md"));
        if let Some(previous) = sequences.insert(&record.sequence, file) {
            let sequence = &record.sequence;
            problems
                .push(format!("record {file}: sequence {sequence} is already used by {previous}"));
        }
        if record_task_id(&record.body) != Some(task_id.as_str()) {
            problems.push(format!(
                "record {file}: first scope line must be \"- Id / capability / checkpoint: `{task_id}` ...\""
            ));
        }
        if !inputs.record_index.contains(&format!("({file})")) {
            problems.push(format!("record {file}: not linked from the records README"));
        }
        if open.contains_key(task_id.as_str()) {
            problems.push(format!("record {file}: task {task_id:?} is still in the plan"));
        }
    }
    records
}

/// A planned capability has open tasks; a shipped one has none.
fn lint_open_work(inputs: &Inputs, problems: &mut Vec<String>) {
    for capability in &inputs.registry {
        let id = &capability.id;
        let count = inputs.plan.tasks.iter().filter(|task| task.capability == *id).count();
        match capability.status.as_str() {
            "planned" if count == 0 => {
                problems.push(format!("registry: planned capability {id:?} has no open tasks"));
            }
            "shipped" if count > 0 => problems.push(format!(
                "registry: shipped capability {id:?} still has {count} open task(s)"
            )),
            _ => {}
        }
    }
}

/// Checks one task; `records` holds record file names without `.md`.
fn lint_task(task: &Task, open: &HashMap<&str, &Task>, records: &HashSet<&str>) -> Vec<String> {
    let mut problems = Vec::new();
    let mut add = |problem: String| {
        problems.push(format!("plan:{}: task {:?}: {problem}", task.line, task.id));
    };
    if task.capability.is_empty() {
        add("not under a capability group heading".to_owned());
    }
    for field in task.lane.required_fields() {
        if task.field(field).is_empty() {
            add(format!("missing field {field:?}"));
        }
    }
    let (status_field, status) = (task.lane.status_field(), task.field(task.lane.status_field()));
    if !status.is_empty() && !task.lane.statuses().contains(&status) {
        add(format!("{status_field} {status:?} is not valid in the {} lane", task.lane));
    }
    let serves = task.field("Serves");
    if !serves.is_empty() && !serves.starts_with(&format!("`{}`", task.capability)) {
        add(format!("Serves must start with `{}`", task.capability));
    }
    for id in task.dependencies() {
        if !open.contains_key(id) {
            add(format!(
                "dependency `{id}` is not an open task (link accepted work as records/...)"
            ));
        }
    }
    for stem in record_links(task.field("Dependencies")) {
        if !records.contains(stem) {
            add(format!("dependency record {stem}.md does not exist"));
        }
    }
    if task.lane == Lane::Human {
        for id in backticked_ids(task.field("Unblocks")) {
            if !open.contains_key(id) {
                add(format!("unblocks `{id}`, which is not an open task"));
            }
        }
    }
    problems
}

/// Returns the task id on the record's `- Id / capability / checkpoint:` line.
fn record_task_id(body: &str) -> Option<&str> {
    body.lines()
        .find_map(|line| line.strip_prefix("- Id / capability / checkpoint: `"))
        .and_then(|rest| rest.split_once('`'))
        .map(|(id, _)| id)
}

/// Returns the record stems (`0001-group-task`) linked as `(records/<stem>.md...)` in `text`.
fn record_links(text: &str) -> Vec<&str> {
    text.match_indices("(records/")
        .filter_map(|(i, prefix)| {
            let (stem, _) = text[i + prefix.len()..].split_once(".md")?;
            let (sequence, rest) = stem.split_at_checked(4)?;
            let valid = sequence.bytes().all(|b| b.is_ascii_digit())
                && rest.strip_prefix('-').is_some_and(is_id);
            valid.then_some(stem)
        })
        .collect()
}

/// Returns one dependency cycle among open tasks, first id repeated at the end, or `None`.
fn find_cycle(tasks: &[Task]) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        Visiting,
        Done,
    }

    fn visit<'a>(
        id: &'a str,
        deps: &HashMap<&'a str, Vec<&'a str>>,
        marks: &mut HashMap<&'a str, Mark>,
        stack: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        match marks.get(id) {
            Some(Mark::Done) => return None,
            Some(Mark::Visiting) => {
                let start = stack.iter().position(|&s| s == id)?;
                let cycle = stack[start..].iter().chain([&id]);
                return Some(cycle.map(|&s| s.to_owned()).collect());
            }
            None => {}
        }
        marks.insert(id, Mark::Visiting);
        stack.push(id);
        for &dep in deps.get(id).into_iter().flatten() {
            if deps.contains_key(dep)
                && let Some(cycle) = visit(dep, deps, marks, stack)
            {
                return Some(cycle);
            }
        }
        stack.pop();
        marks.insert(id, Mark::Done);
        None
    }

    let deps: HashMap<&str, Vec<&str>> =
        tasks.iter().map(|task| (task.id.as_str(), task.dependencies())).collect();
    let (mut marks, mut stack) = (HashMap::new(), Vec::new());
    tasks.iter().find_map(|task| visit(&task.id, &deps, &mut marks, &mut stack))
}
