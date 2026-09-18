//! Open-work summary: task counts and the tasks that may start now.

use std::collections::HashSet;
use std::fmt;

use super::{Lane, Plan, Task};

/// Summary of the open plan.
#[derive(Debug)]
pub struct Status<'a> {
    pub agent_tasks: usize,
    pub human_tasks: usize,
    /// Agent tasks with no open dependency, in plan order.
    pub eligible: Vec<&'a Task>,
    /// Human tasks with no open dependency.
    pub waiting_human: Vec<&'a Task>,
}

impl<'a> Status<'a> {
    /// Derives counts and eligibility. A dependency on any open task, agent or human, blocks;
    /// accepted records never block.
    pub fn new(plan: &'a Plan) -> Self {
        let open: HashSet<&str> = plan.tasks.iter().map(|task| task.id.as_str()).collect();
        let mut status =
            Self { agent_tasks: 0, human_tasks: 0, eligible: vec![], waiting_human: vec![] };
        for task in &plan.tasks {
            let ready = !task.dependencies().iter().any(|id| open.contains(id));
            let (count, ready_list) = match task.lane {
                Lane::Agent => (&mut status.agent_tasks, &mut status.eligible),
                Lane::Human => (&mut status.human_tasks, &mut status.waiting_human),
            };
            *count += 1;
            if ready {
                ready_list.push(task);
            }
        }
        status
    }
}

impl fmt::Display for Status<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (agent, human) = (self.agent_tasks, self.human_tasks);
        writeln!(f, "open tasks: {} ({agent} agent, {human} human)", agent + human)?;
        match self.eligible.split_first() {
            None => writeln!(f, "next agent task: none eligible")?,
            Some((next, others)) => {
                let status = next.field("Agent status");
                writeln!(f, "next agent task: {} ({}, {status})", next.id, next.capability)?;
                if !others.is_empty() {
                    let ids: Vec<&str> = others.iter().map(|task| task.id.as_str()).collect();
                    writeln!(f, "also eligible: {}", ids.join(", "))?;
                }
            }
        }
        for task in &self.waiting_human {
            writeln!(f, "human action available: {} ({})", task.id, task.field("Human status"))?;
        }
        Ok(())
    }
}
