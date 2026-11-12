---
name: review-task
description: Review a task in the kanban board's review column - check its PR against the task, run the checks and a code review, then send it back with notes or mark it ready to merge. Use when asked to review a task or PR, go through the review column, or merge reviewed work.
---

# Review a task

Arguments, optional: a task id (`orch#12`). Without one, take the oldest task in review that has a PR link and isn't tagged `approved` (`list_tasks` with `status: review`).

## 1. Read it

- `get_task` for the description and every comment. The description is the spec; the worker's last comment says what it did.
- If the task is tagged `stuck`, don't review code. Read what blocks it, answer it if you can, or summarise the question for the user. Stop there.
- Find the milestone from its `m<n>` tag. Check the milestone gate task's "done when" test, so you can tell whether this task moves toward it.

## 2. Check it

In a separate worktree, so the main checkout and the board stay untouched:

```
git fetch origin task/orch-<n>
git worktree add ../task-orchestrator-wt/review-orch-<n> origin/task/orch-<n>
```

Then:

1. Run what CI runs: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.
2. Run the `code-review` skill on the PR at medium effort.
3. Check the diff against the task: does it do what the description asks, does it have tests for it, and does it stay in scope? Our code should sit in our own files with small hooks in kanzo's, and our migrations in our own list.

Remove the review worktree when you're done: `git worktree remove ../task-orchestrator-wt/review-orch-<n>`.

## 3. Decide

- **Changes needed:** post the findings on the PR with `gh pr review --request-changes`, then `add_comment` on the task with the same points as a short numbered list. Tag it `changes-requested` and move it to `todo`, so the next `work-task` picks it up and continues the same branch.
- **Ready:** `gh pr review --approve` (or a comment if GitHub refuses approving your own PR), `add_comment` saying it's ready to merge and what you checked, and add the tag `approved`. Leave it in review for the user.

Merging and moving to done are the user's. Only if the user asks you to merge: `gh pr merge --squash --delete-branch`, then remove the task's worktree, clear the `approved` and `changes-requested` tags, and move the task to `done`. That unblocks the tasks waiting on it.
