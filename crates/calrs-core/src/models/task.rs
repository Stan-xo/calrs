//! Task model: a to-do item with optional deadline and priority scoring.

use chrono::{DateTime, Utc};

/// Progression state of a task.
pub enum TaskState {
    Todo,
    Doing,
    Done,
}

/// User-defined criticality level of a task.
///
/// Used as one input to the dynamic priority score,
/// alongside deadline proximity and active task load.
/// Note: `Blocked` tasks are excluded from priority scoring
/// since their deadline is suspended.
pub enum Criticality {
    Low,
    Medium,
    High,
    /// Blocks other work — must be resolved as soon as possible.
    Urgent,
    /// Blocked by an external dependency (technical, material, third-party).
    /// Deadline is suspended, excluded from priority scoring.
    Blocked,
}

/// A to-do item with optional deadline and user-defined criticality.
///
/// Priority is never stored — it is computed dynamically at display time
/// from `criticality`, `deadline` proximity, and active task load.
pub struct Task {
    pub deadline: Option<DateTime<Utc>>,
    pub state: TaskState,
    pub criticality: Option<Criticality>,
}