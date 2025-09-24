# NOW: Scoring and Persistence

This document outlines the immediate development focus for the `reev` framework. The current goal is to implement **Phase 8: Scoring and Persistence** as defined in `PLAN.md`.

## Current Objective

The primary objective is to add a database layer to the `reev-runner` to permanently store and score the results of each evaluation run. This will provide a robust mechanism for tracking and comparing the performance of the LLM agent over time.

## Action Plan

The following tasks from `TASKS.md` are the current priority:

1.  **Task 8.1: Add Turso Dependency**: Integrate the `turso` crate into the `reev-runner` to enable local database functionality.
2.  **Task 8.2: Implement Database Module**: Create a new `db.rs` module within the `reev-runner` to manage the database connection, schema creation (`CREATE TABLE`), and insertion logic.
3.  **Task 8.3: Implement Scoring Logic**: Develop the function that compares the final on-chain state against the `final_state_assertions` from the benchmark file to produce a definitive pass/fail score.
4.  **Task 8.4: Integrate Persistence**: Wire up the main evaluation loop in `reev-runner/src/main.rs` to call the scoring logic and use the database module to save the complete result of the run.
