RTIC Scope application example
---

This repository is an application example of [RTIC Scope](https://github.com/rtic-scope) on a Microchip ATSAME51N20A MCU running a trivial [RTIC](https://rtic.rs) application.
The application configures tracing via `cortex_m_rtic_trace::{configure, trace}` and pends a `app::hardware` task.
This hardware task spawns a software task, `app::software`, with higher priority that instantly exits.
Both tasks are traced.

## Installing RTIC Scope

## Recording a trace with `cargo rtic-scope trace`

## Replaying a trace with `cargo rtic-scope replay`

## Graphically plotting the recorded trace with the `feat/auto-plot` fork
