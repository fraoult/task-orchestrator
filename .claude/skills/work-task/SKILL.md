---
name: work-task
description: Take the next ready task from the kanban board and work it end to end in its own git worktree, then open a PR and move it to review. Use when asked to work the board, take or pull the next task, work on a task id like orch#12, or keep going on a milestone.
---

# Work a task

One run works one task, from pulling it to a PR in review. Several sessions can run this at once, since each gets its own task and worktree. Arguments, all optional: a task id (`orch#12`), or a filter (`tag:m1`, `tag:epic-claims`).

## 1. Get a task

- With a task id: `get_task` it. If it's in todo, move it to doing with `update_task`. If it's in doing or review and someone else is on it, stop and tell the user.
- Otherwise: `next_task` with the repo's project and the filter. It takes the top unblocked, non-human todo task and moves it to doing in one step, so no other agent can get it. If it returns nothing, report how many tasks are waiting and on what, and stop.
- Read the whole task with `get_task`, comments included. If the comments show it came back from review, the work so far is on the branch named there: continue it, and address each review point.

## 2. Set up the worktree

From the main checkout, with `<n>` the task number:

```
git fetch origin main
git worktree add ../task-orchestrator-wt/orch-<n> -b task/orch-<n> origin/main
```

If the branch already exists (a task back from review), use `git worktree add ../task-orchestrator-wt/orch-<n> task/orch-<n>` instead. Do all the work in that folder, using absolute paths. Never build or edit in the main checkout: the board runs from its `target\release\kanzo.exe`.

Comment on the task: which worktree and branch you're working in.

## 3. Do the work

- Stay inside the task. If you find other work that's needed, create a new todo task for it (with tags copied from this one, and `waits_on` if it blocks this task) instead of doing it here.
- Follow the repo's conventions. Our code goes in new files where possible, such as `src/orchestrator.rs`, with small hooks in kanzo's files, and our schema changes go in our own migration list, never kanzo's.
- Add or update tests for the behaviour the task describes.
- Comment on the task when you make a decision a reviewer would want to know about, or hit something surprising. Keep comments short.
- Before finishing, run what CI runs, in the worktree:

```
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## 4. Hand it to review

1. Commit with a message that names the task, like `Record the claimer in next_task (orch#5)`. Push with `git push -u origin task/orch-<n>`.
2. Open a PR against `main` with `gh pr create`, titled like the task, with the task id and a short summary in the body. If the PR already exists, push to it and comment there on what changed.
3. `update_task` with the PR link as `url`.
4. `add_comment` with what you did, what you tested, anything left open, and how to resume: `cd <worktree path>` then `claude --continue`.
5. `update_task` to `review`.

## If you get stuck

If you can't finish (a decision only the user can make, a failing test you can't explain, missing access), push what you have, comment what blocks you and what you tried, add the tag `stuck`, and move the task to review. Don't leave a task sitting in doing.

Never move a task to done, and never take a human task. Those belong to the user.
