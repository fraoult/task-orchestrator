---
name: plan-to-board
description: Turn a plan (a doc, a chat outline, or a Markdown file) into the board's plan format and load it onto the kanban board as tasks with tags and dependencies. Use when asked to plan work, break a plan into tasks, load or import a plan or milestone onto the board, or update the board from a changed plan.
---

# Plan to board

Turns a plan into epics, milestones and tasks on the orchestrator board (the `mcp__orchestrator__*` tools). It works in two steps that you can stop between: first write the plan file, then load it.

The plan file format is in [plan-format.md](plan-format.md). Read it before writing a plan file. The same format is what M4's `import_plan` will read, so keep files strictly to it.

## Step 1: write the plan file

1. Get the source plan. It may be a Claude Docs link (read it with the Claude Docs connector), a file path, or text in the chat.
2. Choose the board project. Run `list_projects`. Use the project linked to this repo; if there is none, ask once whether to create one, suggesting a short slug.
3. Break each milestone into tasks of about one pull request each: one agent, one branch, a reviewable diff. A task's description must let an agent with no other context start work. It says what to change, where (files or modules when known), and how to tell it's finished (a test to add, a command to run).
4. Mark as `[human]` any task that needs the user: decisions, accounts, installs, hardware, manual checks.
5. Add dependencies only where one task really needs another's code or decision. Leave tasks that can run in parallel independent, so several agents can work at once.
6. Write the file to `docs/plans/<plan-slug>.md`. If the file exists, update it in place and keep the board ids already recorded in it.
7. Show the user a short preview: milestones, task counts, human tasks, and anything you were unsure about. Wait for their go-ahead before Step 2, unless they already asked you to load it.

## Step 2: load it onto the board

Until the board has real epics and milestones (planned for M2), map them like this:

| Plan | Board today |
|---|---|
| Epic `{claims}` | tag `epic-claims` on each of its tasks |
| Milestone `M1` | tag `m1` on each of its tasks |
| Priority `[p1]` | tag `p1` |
| `[human]` | `human: true` |
| Milestone gate | one human task per milestone, titled `M1 complete: <milestone title>`, tagged with the milestone, its epic and `gate` |
| A task or milestone waits on milestone M1 | it waits on M1's gate task |

The gate task is how milestones get their "done when" test: it waits on every task in the milestone, its description is the milestone's `Done when:` test, and the user moves it to done after checking it. That unblocks everything that waits on the milestone.

Load order matters, because `next_task` takes the top unblocked todo card in board order:

1. Create tasks milestone by milestone in plan order. Within a milestone, create higher priority first, and a task before the tasks that wait on it.
2. Create each task with `create_task`: `project`, `title`, `description`, `tags` (comma-separated), `human`, and `waits_on` for dependencies that already exist on the board.
3. After a milestone's tasks exist, create its gate with `waits_on` listing all of them. Then set `waits_on` on the next milestone's tasks to include the gate wherever that milestone waits on this one. A task can wait on several tasks, comma-separated, like `orch#3, orch#7`.
4. Record each board id back into the plan file right after creating it, as a trailing `<!-- orch#12 -->` on the task's line, and each gate's id on a `gate: <!-- orch#9 -->` line under the milestone's `done when:`. This makes re-runs safe: a task line that already has an id is updated with `update_task`, never created again.
5. Never delete board tasks because they vanished from the plan. List them for the user and let them decide.

Finish by running `list_tasks` for the project and showing the user what is ready now, what is blocked, and the human tasks waiting on them.

## When the board gains epics and milestones

Once `create_epic`, `create_milestone` and `set_waits` exist (M2), or `import_plan` (M4), use those instead of tags and gate tasks, and update this skill.
