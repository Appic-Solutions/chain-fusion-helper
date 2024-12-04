use serde::{Deserialize, Serialize};

use crate::state::mutate_state;

#[derive(Clone, PartialEq, Hash, Debug, PartialOrd, Eq, Ord, Deserialize, Serialize, Copy)]
pub enum TaskType {
    RemoveUnverified,
    ScrapeEvents,
    UpdateTokenPairs,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct TimerGuard {
    task: TaskType,
}
#[derive(Debug, PartialEq, Eq)]
pub enum TimerGuardError {
    AlreadyProcessing,
}

impl TimerGuard {
    pub fn new(task: TaskType) -> Result<Self, TimerGuardError> {
        mutate_state(|s| {
            if !s.active_tasks.insert(task) {
                return Err(TimerGuardError::AlreadyProcessing);
            }
            Ok(Self { task })
        })
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.active_tasks.remove(&self.task);
        });
    }
}
