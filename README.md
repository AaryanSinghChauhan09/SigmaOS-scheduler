# SigmaOS Scheduler

Rust models for bounded run queues, process scheduling state, CPU affinity,
NUMA placement, deadline policies, and scheduler configuration. The exported
`Scheduler` and `SchedulerCore` APIs manage in-memory task and queue state.

## Runtime limits

These data structures do not by themselves context-switch CPUs, program a
kernel timer, enforce real-time deadlines, or install an eBPF scheduler. The
module names describe intended areas; they are not evidence of Linux kernel
integration or measured latency and fairness.

Record scheduling invariants and benchmark evidence for changes affecting
latency, fairness, power, or interactive responsiveness. Validate kernel-facing
behavior separately on an identified target and kernel version.

## Development

The crate depends on `SigmaOS-kernel` through a sibling path dependency. CI
resolves SigmaOS path dependencies and runs `cargo test --all-targets --locked`.
