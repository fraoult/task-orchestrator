# Task orchestrator
project: orch

Our fork of kanzo, grown into a central task orchestrator: a plan becomes a graph of work, and agents pull the right next task from it. The full plan, with the reasoning, is the "Task Orchestrator: initial plan" doc. M0 (fork and set up Rust) is done.

Conventions for every task: our code goes in new files where possible, such as `src/orchestrator.rs`, with small hooks in kanzo's files. Our schema changes go in our own migration list, never kanzo's `MIGRATIONS`. Keep kanzo's style of short plain-text MCP replies.

## M1: Statuses and claims {claims}
waits on: none
done when: Two agents race for the same task and only one gets it, and a killed agent's task comes back to todo after the timeout.
gate: <!-- orch#9 -->

Keep kanzo's todo, doing, review and done. Record who claimed each task, and add heartbeats, lease expiry, `release_task` and the events log.

1. **Add our own migration list and claim columns** [p1] <!-- orch#1 -->
   Create `src/orchestrator.rs` with an `ORCH_MIGRATIONS` list whose version is tracked separately from kanzo's (for example its own key in `meta`), and run it from the store's open path right after kanzo's migrations. First migration: add `claimed_by`, `claimed_at`, `heartbeat_at` and `worktree` to `tasks`, and create an append-only `events` table (id, task_id, kind, actor, detail, at). Tests: it applies to a fresh database and to an existing kanzo database, and running it twice changes nothing.
2. **Record the claimer in next_task** [p1] (waits: 1) <!-- orch#2 -->
   Give `next_task` optional `agent` and `worktree` arguments (agent defaults to the MCP client's name). Set `claimed_by`, `claimed_at`, `heartbeat_at` and `worktree` in the same locked write that moves the task to doing. Moving a task out of doing by any route clears the claim. Show the claimer in `get_task` and in `list_tasks` lines. Tests in `store.rs` style.
3. **Write claims and status changes to the events log** [p1] (waits: 1) <!-- orch#3 -->
   Log an event for each claim, status change, release and expiry, with the actor. Add a short history section to `get_task` output (last few events). Tests.
4. **Add the heartbeat tool** [p1] (waits: 2) <!-- orch#4 -->
   MCP tool `heartbeat(id, agent?)` that refreshes `heartbeat_at` on a task in doing. Refuse it, with a clear message, if another agent holds the claim or the task isn't in doing. Tests.
5. **Expire stale claims** [p1] (waits: 3, 4) <!-- orch#5 -->
   Add `lease_timeout` to `config.toml`, default one hour. A sweep returns doing tasks whose `heartbeat_at` is older than the timeout to todo, clears the claim, logs an `expired` event and comments on the task. Run the sweep at the start of each `next_task` write and on a timer in the board server. Tasks moved to doing by hand in the board (no claim) are never expired. Tests with a short timeout.
6. **Test the race and the killed agent** [p1] (waits: 2, 5) <!-- orch#6 -->
   Integration tests: several threads, each with its own store connection, call `next_task` on one task at once and exactly one gets it. A claimed task with no heartbeats comes back to todo after the timeout and can be claimed again.
7. **Add the release_task tool** [p2] (waits: 2, 3) <!-- orch#7 -->
   MCP tool `release_task(id, reason?)` that puts a doing task back in todo at the top of the column, clears the claim, logs an event, and adds the reason as a comment. Tests.
8. **Teach work-task to heartbeat and release** [p2] (waits: 4, 7) <!-- orch#8 -->
   Update `.claude/skills/work-task/SKILL.md`: pass `agent` and `worktree` to `next_task`, call `heartbeat` between steps of long work, and `release_task` instead of leaving a task in doing when giving up.

## M2: Epics, milestones and priority {planning}
waits on: M1
done when: A task waiting on a milestone stays blocked until every task in that milestone is done.
gate: <!-- orch#17 -->

1. **Add the epics and milestones tables** [p1] <!-- orch#10 -->
   Our migration: `epics` (id, project_id, title, description, priority, position) and `milestones` (id, project_id, epic_id optional, title, description, position, target_date optional), plus `epic_id` and `milestone_id` on tasks. Store functions to create, list, update and delete them. Tests.
2. **Add task priority** [p1] (waits: 1) <!-- orch#11 -->
   Our migration: `priority` on tasks, 0 to 3, default 2. Accept it in create and update, and show it as `P0` to `P3` in task lines. Tests.
3. **Add milestone dependencies** [p1] (waits: 1) <!-- orch#12 -->
   Our migration: `milestone_waits`, where a task or a milestone waits on a milestone. A milestone is finished once every task in it is done; its status is worked out from its tasks, never set by hand. Extend the waiting check used by `next_task` and `is:waiting` to include unfinished milestones. Refuse cycles. Tests, including the M2 done-when case.
4. **Add the planning MCP tools** [p1] (waits: 2, 3) <!-- orch#13 -->
   `create_epic`, `create_milestone` and `set_waits` (on tasks or milestones, for tasks or milestones), and `epic`, `milestone` and `priority` arguments on `create_task` and `update_task`. Short plain-text replies. Tests.
5. **Add epic:, milestone: and priority: filters** [p2] (waits: 1, 2) <!-- orch#14 -->
   Extend the filter language in `store.rs`, and the highlighting in `app.js`. Tests alongside the existing filter tests.
6. **Show epics, milestones and priority on the board** [p2] (waits: 4) <!-- orch#15 -->
   In `app.js` and `index.html`: group or label cards by milestone and epic, show a priority badge, and dim cards waiting on a milestone like cards waiting on a task.
7. **Switch plan-to-board to real epics and milestones** [p2] (waits: 4) <!-- orch#16 -->
   Update `.claude/skills/plan-to-board/SKILL.md` to create epics, milestones and milestone waits instead of tags and gate tasks, and migrate the tasks already on the board.

## M3: Smarter next task {scheduling}
waits on: M2
done when: With a goal set, agents only take tasks on the path to it, in priority order.
gate: <!-- orch#22 -->

1. **Order next_task by milestone, priority and unblocking** [p1] <!-- orch#18 -->
   Among ready tasks, prefer the earliest open milestone, then higher priority, then tasks that unblock the most other work (directly or through other tasks), then board position. Tests for each tie-break.
2. **Add a goal to next_task** [p1] (waits: 1) <!-- orch#19 -->
   Optional `goal` argument, a milestone or a task: keep only tasks the goal depends on, directly or through other tasks and milestones. Tests.
3. **Add the graph tool** [p2] (waits: 2) <!-- orch#20 -->
   MCP tool `graph(goal?)` returning the ready and blocked sets for a goal, with what each blocked task waits on, in short plain text. Tests.
4. **Teach work-task to pass a goal** [p3] (waits: 2) <!-- orch#21 -->
   Update `.claude/skills/work-task/SKILL.md` to accept a milestone as the goal, and `board-status` to use `graph`.

## M4: Plan import {import}
waits on: M2
done when: One of your real plans becomes epics, milestones and tasks with correct dependencies in one step.
gate: <!-- orch#28 -->

The format is `.claude/skills/plan-to-board/plan-format.md`, version 0. Refine it as we go, and keep the skill and the parser in step.

1. **Parse the plan format** [p1] <!-- orch#23 -->
   `src/plan.rs`: parse a plan file into milestones, epics and tasks with their keys, markers, descriptions and waits, and report errors with line numbers. Test against `docs/plans/task-orchestrator.md` and small hand-written cases.
2. **Preview an import** [p1] (waits: 1) <!-- orch#24 -->
   Compare a parsed plan with the board: what would be created, updated and left alone, matched by the recorded board ids, then by key. Nothing is written. Tests.
3. **Add import_plan as a tool and a command** [p1] (waits: 2) <!-- orch#25 -->
   MCP tool `import_plan(path, apply?)` and `kanzo import <file> [--apply]`. Without apply it shows the preview; with it, it writes everything in one transaction and records board ids back into the file. Re-importing creates no duplicates. Tests.
4. **Try the import on one of your real plans** [human] (waits: 3) <!-- orch#26 -->
   Pick a real plan, run it through `plan-to-board` and `import_plan`, and note where the format or the result felt wrong as comments on this task.
5. **Switch plan-to-board to import_plan** [p2] (waits: 3) <!-- orch#27 -->
   Update the skill to write the file, then preview and apply with `import_plan` instead of creating tasks one by one.

## M5: Graph view {graph-view}
waits on: M3
done when: You can see the dependency graph on the board and add or remove a dependency by dragging.
gate: <!-- orch#32 -->

1. **Serve the graph to the board** [p2] <!-- orch#29 -->
   An endpoint in `web.rs` returning tasks, milestones and their waits for a project, reusing the store's graph code from M3. Tests.
2. **Draw the dependency graph** [p2] (waits: 1) <!-- orch#30 -->
   A graph view in `app.js` and `index.html`, laid out left to right by dependency, coloured by status, that updates live like the board.
3. **Edit dependencies by dragging** [p2] (waits: 2) <!-- orch#31 -->
   Drag from one node to another to add a wait, and remove one from its edge. Refused cycles show the server's message.

## M6: Central host {hosting}
waits on: M3
done when: The board runs as a service on this PC, and several agents work one milestone in separate worktrees without stepping on each other.
gate: <!-- orch#35 -->

1. **Run the board as a service on this PC** [p2] <!-- orch#33 -->
   Check what `service.rs` already does on Windows, and make the board start with the machine and survive restarts, with the lease sweep running. Document it in the README.
2. **Run several agents on one milestone** [human] (waits: 1) <!-- orch#34 -->
   Start three sessions running `work-task` on the same milestone, let them work, and note any friction as comments here. That feedback shapes what comes after M6.

## M7: Push dispatcher {dispatcher}
waits on: M6
done when: A dispatcher starts agents in fresh worktrees for ready tasks, with no manual pulls.
gate: <!-- orch#38 -->

Optional. Only once the pull model works in daily use.

1. **Decide whether to build the dispatcher** [human] [p3] <!-- orch#36 -->
   After some time on M6, decide whether a dispatcher is worth it, and how many agents it may run at once.
2. **Build the push dispatcher** [p3] (waits: 1) <!-- orch#37 -->
   A small separate program that watches for ready tasks and starts a Claude Code session per task in a fresh worktree, running `work-task` with that task's id, up to a set number at once. It uses the same MCP tools as any agent.
