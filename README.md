# SigmaOS Scheduler

> CPU scheduling implementation for SigmaOS inspired by Linux kernel's Completely Fair Scheduler (CFS) and Earliest Eligible Virtual Deadline First (EEVDF)

## Overview

This component implements CPU scheduling for SigmaOS, providing multiple scheduling policies including Completely Fair Scheduler (CFS), Earliest Eligible Virtual Deadline First (EEVDF), real-time scheduling (SCHED_FIFO, SCHED_RR), and deadline scheduling (SCHED_DEADLINE). It enables efficient CPU resource allocation with fairness guarantees and priority-based preemption.

## Features

- Completely Fair Scheduler (CFS) implementation
- Earliest Eligible Virtual Deadline First (EEVDF) scheduler
- Real-time scheduling policies (SCHED_FIFO, SCHED_RR)
- Deadline scheduling (SCHED_DEADLINE)
- Priority-based preemption
- CPU affinity support
- NUMA-aware scheduling
- Load balancing across CPUs
- Scheduler classes and policies
- vruntime-based fair scheduling
- Red-black tree task queues

## Architecture

### Scheduler Classes

```rust
pub trait SchedulerClass {
    fn enqueue_task(&mut self, task: &Task);
    fn dequeue_task(&mut self, task: &Task);
    fn pick_next_task(&mut self) -> Option<TaskId>;
    fn task_tick(&mut self, task: &Task);
    fn check_preempt_curr(&self, task: &Task) -> bool;
}

pub enum SchedulerPolicy {
    Normal,      // CFS/EEVDF
    RealTimeFifo, // SCHED_FIFO
    RealTimeRoundRobin, // SCHED_RR
    Deadline,    // SCHED_DEADLINE
    Idle,        // Idle tasks
}
```

### Task Structure

```rust
pub struct Task {
    pub id: TaskId,
    pub pid: u32,
    pub priority: i32, // nice value (-20 to 19)
    pub vruntime: u64,
    pub deadline: u64,
    pub runtime: u64,
    pub period: u64,
    pub policy: SchedulerPolicy,
    pub affinity: CpuSet,
    pub state: TaskState,
}

pub struct TaskState {
    pub runnable: bool,
    pub sleeping: bool,
    pub stopped: bool,
    pub zombie: bool,
}
```

### CPU Run Queue

```rust
pub struct RunQueue {
    pub tasks: BTreeMap<u64, TaskId>, // vruntime -> task_id
    pub current_task: Option<TaskId>,
    pub load: LoadInfo,
    pub nr_running: usize,
    pub min_vruntime: u64,
}

pub struct LoadInfo {
    pub runnable: usize,
    pub weight: u64,
    pub avg_load: f32,
}
```

## Installation

```bash
# Clone the repository
git clone https://github.com/AaryanSinghChauhan09/SigmaOS-scheduler.git

# Build
cd SigmaOS-scheduler
cargo build --release
```

## Usage

### Initialize Scheduler

```rust
use sigmaos_scheduler::*;

fn main() -> Result<()> {
    let mut scheduler = Scheduler::new(num_cpus)?;
    scheduler.init()?;
    
    // Create tasks
    let task1 = Task::new(1000, SchedulerPolicy::Normal, 0);
    let task2 = Task::new(1001, SchedulerPolicy::RealTimeFifo, 10);
    
    // Add tasks to scheduler
    scheduler.enqueue_task(task1)?;
    scheduler.enqueue_task(task2)?;
    
    // Run scheduler
    loop {
        if let Some(task_id) = scheduler.pick_next_task()? {
            scheduler.run_task(task_id)?;
        }
    }
}
```

### Set Task Priority

```rust
// Set nice value (-20 to 19, lower is higher priority)
scheduler.set_priority(task_id, -5)?;

// Set real-time priority (1-99)
scheduler.set_rt_priority(task_id, 50)?;

// Set deadline parameters
scheduler.set_deadline(task_id, runtime_ns, period_ns, deadline_ns)?;
```

### CPU Affinity

```rust
// Set CPU affinity mask
let mut affinity = CpuSet::new();
affinity.set_cpu(0);
affinity.set_cpu(1);
scheduler.set_affinity(task_id, affinity)?;

// Get allowed CPUs
let allowed = scheduler.get_affinity(task_id)?;
```

### Load Balancing

```rust
// Trigger load balancing
scheduler.balance_load()?;

// Get load information
let load = scheduler.get_load(cpu_id)?;
println!("CPU {} load: {} runnable tasks", cpu_id, load.runnable);
```

## Scheduling Policies

### Normal (CFS/EEVDF)

- Fair CPU time distribution
- Nice value-based priority
- vruntime-based scheduling
- Red-black tree task queue
- Automatic load balancing

### Real-Time FIFO (SCHED_FIFO)

- Strict priority ordering
- No time slicing
- Preempts all lower priority tasks
- Runs until blocked or preempted

### Real-Time Round-Robin (SCHED_RR)

- Priority-based with time slicing
- Equal time slices for same priority
- Preempts lower priority tasks
- Rotates among same priority tasks

### Deadline (SCHED_DEADLINE)

- Earliest deadline first
- Runtime, period, deadline parameters
- Admission control
- Preempts all other tasks

## Integration with SigmaOS

- Works with process management
- Integrates with interrupt handling
- Supports NUMA architectures
- Works with real-time applications

## Requirements

- Multi-core CPU support
- Kernel space execution
- High-resolution timers
- NUMA awareness (optional)

## Development

See [SigmaOS Development Guide](https://github.com/AaryanSinghChauhan09/SigmaOS/wiki/development) for general guidelines.

## Testing

```bash
# Run unit tests
cargo test

# Run scheduler tests
cargo test --test scheduler

# Run load balancing tests
cargo test --test load_balancing

# Run real-time tests
cargo test --test realtime
```

## License

MIT OR GPL-2.0 — Free and Open Source Software.
© SigmaOS Contributors
