# ADR 0002: lower the scheduling priority of background job threads in `crates/platform`

- **Status:** accepted (2026-10-30, approved by the project owner; extends the scope of
  [ADR 0001](0001-platform-ffi.md), whose containment rules apply unchanged)

## Context

An export renders on every core. Exports, the export queue, Render Previews and proxies ran at the
same priority as everything else, so when the machine was busy the operating system could not know
that the export can wait and the monitors cannot.

Measured on an M4 Max (2026-10-30, release builds, a 4K60 H.264 export while the Program monitor
played another 4K60 clip): with the job's threads at the default priority the monitor's decode p95
was 17–22 ms; at the utility class 3.6 ms. Playback dropped no frames either way on that machine,
which had cores to spare; on a busier or smaller one the difference is frames. An export alone took
about the same time (89.4–90.9 s at the default priority, 90.4–94.5 s at utility).

Two things the monitors shared with background jobs are fixed without any `unsafe`: the rendering
threads (background jobs run on a pool of their own, `filmcraft_export::on_own_threads`) and the
decoders (a job reads its own copy of each file, `MediaPool::own_source`; that was why playing a
clip while it exported dropped 93 % of the frames). Telling the system which threads can wait is
an OS call (on macOS `pthread_set_qos_class_self_np`), which needs `unsafe` Rust. ADR 0001 allows
`unsafe` only for OS media FFI and says anything else needs a new decision: this is that decision.

## Decision

`crates/platform` may also call the OS to lower the scheduling priority of the calling thread,
for the threads of background jobs, in one FFI module (`thread_priority`), under ADR 0001's
containment rules (`#[allow(unsafe_code)]` on that module only, a `// SAFETY:` comment on every
block, a safe public API, nothing that can unwind across the boundary).

- **How it is wired:** the export crate runs a hook on each thread of a background job's pool as
  it starts (`filmcraft_export::register_background_thread_start`); `filmcraft_platform::register`
  installs `thread_priority::lower_current_thread` there. Lower layers do not depend on the
  platform crate, and without it (tests, the browser) nothing changes.
- **macOS:** the utility quality-of-service class. The monitors' frame workers stay at the default
  class and the main thread at user-interactive. A thread a utility thread creates starts at the
  default class again (macOS does not pass the class on; tested), so decoders an export opens,
  which the monitors later share, keep their priority.
- **Windows and Linux** follow as their own changes under this decision. Windows thread priorities
  are not inherited either (`SetThreadPriority`). Linux `nice` values are inherited by new threads,
  so a Linux version must also keep the pools of decoders an export opens at normal priority.

## Consequences

- When the machine is busy, the monitors and the UI get the cores first and the export takes
  longer; when it is idle, the export still uses every core.
- The only new `unsafe` is a call that changes the calling thread's own scheduling attributes;
  it reads and writes no memory of ours.
- Anything else that wants `unsafe` still needs a new decision.
