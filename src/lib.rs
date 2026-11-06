//! SigmaOS-scheduler — Linux CFS-inspired task scheduler.
//!
//! Ports the core ideas of `kernel/sched/fair.c`:
//! - `task_struct` analogue: [`Task`] with a strict state machine
//! - Runqueue ordered by *virtual runtime* (vruntime) in a red-black tree,
//!   so the task that has run the least always executes next — fairness
//!   by construction, no starvation (unlike FIFO).
//! - Weighted vruntime: nice-level priorities scale how fast vruntime
//!   accrues (Linux `sched_prio_to_weight` analogue).

use std::collections::BTreeMap;

/// Linux `task_struct->state` analogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// On a runqueue, waiting for CPU (Linux TASK_RUNNING + on_rq).
    Ready,
    /// Currently executing on a CPU.
    Running,
    /// Waiting for I/O, lock, or sleep (TASK_INTERRUPTIBLE/TASK_UNINTERRUPTIBLE).
    Blocked,
    /// Terminated, awaiting reap by parent (EXIT_ZOMBIE).
    Zombie,
}

/// Scheduler error — fail-fast on invalid transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedError {
    NoSuchTask(u64),
    DuplicatePid(u64),
    InvalidTransition { from: TaskState, to: TaskState },
    TaskNotReady(u64),
    TaskNotBlocked(u64),
}

impl std::fmt::Display for SchedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedError::NoSuchTask(p) => write!(f, "no such task: {p}"),
            SchedError::DuplicatePid(p) => write!(f, "duplicate pid: {p}"),
            SchedError::InvalidTransition { from, to } => {
                write!(f, "invalid state transition: {from:?} -> {to:?}")
            }
            SchedError::TaskNotReady(p) => write!(f, "task {p} is not ready"),
            SchedError::TaskNotBlocked(p) => write!(f, "task {p} is not blocked"),
        }
    }
}
impl std::error::Error for SchedError {}

/// `task_struct` analogue.
#[derive(Debug, Clone)]
pub struct Task {
    pub pid: u64,
    pub comm: String,
    pub state: TaskState,
    /// Virtual runtime — smaller means "has run less", wins the CPU.
    pub vruntime: u64,
    /// Scheduling weight (nice -20..19 maps to weight; default 1024 like Linux NICE_0_LOAD).
    pub weight: u64,
    /// CPU time actually consumed, in ticks.
    pub cpu_ticks: u64,
}

impl Task {
    pub fn new(pid: u64, comm: impl Into<String>) -> Self {
        Self {
            pid,
            comm: comm.into(),
            state: TaskState::Ready,
            vruntime: 0,
            weight: 1024,
            cpu_ticks: 0,
        }
    }

    /// `set_user_nice` analogue: lower nice = higher weight = slower vruntime growth.
    pub fn set_nice(&mut self, nice: i32) {
        let clamped = nice.clamp(-20, 19);
        // weight ~= 1024 * 1.25^(-nice); higher weight => slower vruntime => more CPU.
        self.weight = ((1024f64 * 1.25f64.powi(-clamped)) as u64).max(1);
    }
}

/// Red-black-tree-ordered runqueue: BTreeMap keyed by (vruntime, pid)
/// gives O(log n) pick-next with identical ordering semantics to Linux's
/// rb-tree of `se` entities (vruntime is the sort key, pid breaks ties).
type Runqueue = BTreeMap<(u64, u64), u64>; // (vruntime, pid) -> pid

/// Per-state task storage.
#[derive(Default)]
pub struct Scheduler {
    tasks: BTreeMap<u64, Task>,       // all live tasks by pid
    ready: Runqueue,                  // TASK_RUNNING on runqueue
    blocked: BTreeMap<u64, TaskState>,// pid -> last blocked state marker
    zombie: Vec<u64>,                 // EXIT_ZOMBIE pids awaiting reap
    current: Option<u64>,             // running task pid
    min_vruntime: u64,                // CFS lag compensation baseline
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// `fork` analogue: register a new task on the runqueue.
    pub fn spawn(&mut self, task: Task) -> Result<(), SchedError> {
        if self.tasks.contains_key(&task.pid) {
            return Err(SchedError::DuplicatePid(task.pid));
        }
        let pid = task.pid;
        self.ready.insert((task.vruntime, pid), pid);
        self.tasks.insert(pid, task);
        Ok(())
    }

    /// CFS `pick_next_task`: leftmost = lowest vruntime.
    pub fn pick_next(&self) -> Option<u64> {
        self.ready.keys().next().map(|&(_, pid)| pid)
    }

    /// Context switch: charge `delta` ticks of CPU to `pid`, then pick next.
    /// Returns the pid that should now run.
    pub fn tick(&mut self, delta: u64) -> Result<u64, SchedError> {
        let pid = self.current.ok_or(SchedError::NoSuchTask(u64::MAX))?;
        self.charge(pid, delta)?;
        self.schedule()
    }

    /// Direct schedule without charging (used after wake/spawn).
    pub fn schedule(&mut self) -> Result<u64, SchedError> {
        // Dequeue current from ready, pick new leftmost, enqueue old.
        if let Some(old) = self.current.take() {
            if let Some(t) = self.tasks.get(&old) {
                if t.state == TaskState::Running {
                    self.ready.insert((t.vruntime, old), old);
                }
            }
        }
        let next = self.pick_next().ok_or(SchedError::NoSuchTask(u64::MAX))?;
        self.ready.remove(&(self.tasks[&next].vruntime, next));
        self.tasks.get_mut(&next).unwrap().state = TaskState::Running;
        self.current = Some(next);
        Ok(next)
    }

    /// Charge CPU time: vruntime += delta * NICE_0_LOAD / weight.
    fn charge(&mut self, pid: u64, delta: u64) -> Result<(), SchedError> {
        let t = self.tasks.get_mut(&pid).ok_or(SchedError::NoSuchTask(pid))?;
        t.cpu_ticks += delta;
        // Fixed-point: vruntime += delta * 1024 / weight
        t.vruntime += delta.saturating_mul(1024) / t.weight.max(1);
        // Keep vruntime near the min to avoid unbounded growth (CFS
        // min_vruntime lag compensation analogue).
        if let Some(&min_key) = self.ready.keys().next() {
            let min_vr = min_key.0;
            if t.vruntime > min_vr + (1 << 20) {
                t.vruntime = min_vr + (1 << 20);
            }
        }
        Ok(())
    }

    /// TASK_INTERRUPTIBLE analogue: remove from runqueue until woken.
    pub fn block(&mut self, pid: u64) -> Result<(), SchedError> {
        let t = self.tasks.get_mut(&pid).ok_or(SchedError::NoSuchTask(pid))?;
        if t.state != TaskState::Ready && t.state != TaskState::Running {
            return Err(SchedError::InvalidTransition { from: t.state, to: TaskState::Blocked });
        }
        t.state = TaskState::Blocked;
        self.ready.remove(&(t.vruntime, pid));
        if self.current == Some(pid) {
            self.current = None;
            self.schedule()?;
        }
        self.blocked.insert(pid, TaskState::Blocked);
        Ok(())
    }

    /// Wake a blocked task back onto the runqueue.
    pub fn wake(&mut self, pid: u64) -> Result<(), SchedError> {
        let t = self.tasks.get_mut(&pid).ok_or(SchedError::NoSuchTask(pid))?;
        if t.state != TaskState::Blocked {
            return Err(SchedError::TaskNotBlocked(pid));
        }
        t.state = TaskState::Ready;
        self.blocked.remove(&pid);
        self.ready.insert((t.vruntime, pid), pid);
        Ok(())
    }

    /// exit() analogue: mark zombie, remove from runqueue.
    pub fn exit(&mut self, pid: u64) -> Result<(), SchedError> {
        let t = self.tasks.get_mut(&pid).ok_or(SchedError::NoSuchTask(pid))?;
        if t.state == TaskState::Zombie {
            return Err(SchedError::InvalidTransition { from: t.state, to: TaskState::Zombie });
        }
        t.state = TaskState::Zombie;
        self.ready.remove(&(t.vruntime, pid));
        self.blocked.remove(&pid);
        if self.current == Some(pid) {
            self.current = None;
        }
        self.zombie.push(pid);
        Ok(())
    }

    /// wait() analogue: reap a zombie, freeing its pid.
    pub fn reap(&mut self, pid: u64) -> Result<(), SchedError> {
        if !self.zombie.contains(&pid) {
            return Err(SchedError::NoSuchTask(pid));
        }
        self.zombie.retain(|&p| p != pid);
        self.tasks.remove(&pid);
        Ok(())
    }

    pub fn state_of(&self, pid: u64) -> Option<TaskState> {
        self.tasks.get(&pid).map(|t| t.state)
    }
    pub fn current(&self) -> Option<u64> {
        self.current
    }
    pub fn ready_len(&self) -> usize {
        self.ready.len()
    }
    pub fn zombie_len(&self) -> usize {
        self.zombie.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn least_vruntime_runs_first() {
        let mut s = Scheduler::new();
        s.spawn(Task::new(1, "a")).unwrap();
        s.spawn(Task::new(2, "b")).unwrap();
        // Both vruntime 0 — pid tiebreak picks 1 first.
        assert_eq!(s.schedule().unwrap(), 1);
    }

    #[test]
    fn cfs_fairness_rotates() {
        let mut s = Scheduler::new();
        s.spawn(Task::new(1, "a")).unwrap();
        s.spawn(Task::new(2, "b")).unwrap();
        let mut order = Vec::new();
        for _ in 0..4 {
            let pid = s.schedule().unwrap();
            s.tick(10).unwrap(); // charge current, auto-reschedule
            order.push(pid);
        }
        // vruntime accrues equally -> tasks alternate.
        assert_eq!(order, vec![1, 2, 1, 2]);
    }

    #[test]
    fn nice_task_gets_less_cpu() {
        let mut s = Scheduler::new();
        let mut nice = Task::new(1, "nice");
        nice.set_nice(19); // lowest priority
        let mut normal = Task::new(2, "normal");
        normal.set_nice(0);
        s.spawn(nice).unwrap();
        s.spawn(normal).unwrap();
        // Run many ticks; normal should accumulate far more vruntime-adjusted time
        // i.e. nice task's vruntime grows faster, so it runs less often.
        nice_runs_share_is_smaller(&mut s);
    }

    fn nice_runs_share_is_smaller(s: &mut Scheduler) {
        let mut counts = [0u64; 3]; // index by pid-1
        for _ in 0..200 {
            let pid = s.schedule().unwrap();
            s.tick(1).unwrap();
            counts[(pid - 1) as usize] += 1;
        }
        assert!(counts[1] > counts[0], "normal task should run more: {counts:?}");
    }

    #[test]
    fn block_wake_cycle() {
        let mut s = Scheduler::new();
        s.spawn(Task::new(1, "a")).unwrap();
        s.spawn(Task::new(2, "b")).unwrap();
        s.block(1).unwrap();
        assert_eq!(s.state_of(1), Some(TaskState::Blocked));
        // Only task 2 can run now.
        assert_eq!(s.schedule().unwrap(), 2);
        s.wake(1).unwrap();
        assert_eq!(s.state_of(1), Some(TaskState::Ready));
    }

    #[test]
    fn exit_and_reap() {
        let mut s = Scheduler::new();
        s.spawn(Task::new(1, "a")).unwrap();
        s.exit(1).unwrap();
        assert_eq!(s.state_of(1), Some(TaskState::Zombie));
        s.reap(1).unwrap();
        assert_eq!(s.state_of(1), None);
    }

    #[test]
    fn invalid_transitions_fail() {
        let mut s = Scheduler::new();
        s.spawn(Task::new(1, "a")).unwrap();
        assert!(matches!(s.wake(1), Err(SchedError::TaskNotBlocked(1))));
        s.exit(99);
        assert!(matches!(s.exit(99), Err(SchedError::NoSuchTask(99))));
    }
}
