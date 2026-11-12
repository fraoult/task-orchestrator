# Plan format

A plan is one Markdown file. Headings and lists carry the structure; everything else is free text that becomes descriptions. Version 0, to be refined along with M4's `import_plan`.

## Example

```markdown
# Task orchestrator
project: orch

Free text about the plan. Ignored by the importer.

## M1: Statuses and claims {claims}
waits on: none
done when: Two agents race for the same task and only one gets it, and a killed agent's task comes back after the timeout.

Free text about the milestone, kept as the gate task's description.

1. **Add claim columns to tasks** [p1] <!-- orch#4 -->
   Add a migration in our own migration list with `claimed_by`, `claimed_at`,
   `heartbeat_at` and `worktree` on tasks. Test that it applies to an existing kanzo database.
2. **Record the claimer in next_task** [p1] (waits: 1)
   ...
3. **Decide the default lease** [human]
   ...

## M2: Epics, milestones and priority {planning}
waits on: M1
done when: ...

1. **Add the epics table** (waits: M1.2)
   ...
```

## Rules

- `# Title` once, then a `project: <slug>` line.
- `## M<n>: <title> {<epic-slug>}` starts a milestone. The `{epic}` part is optional and names the epic all its tasks belong to. A task can override it with its own `{epic}`.
- Right under the milestone heading, two lines:
  - `waits on:` with `none`, or milestone ids like `M1, M3`.
  - `done when:` with one sentence the user can check. It becomes the gate task.
  - `gate: <!-- orch#9 -->`, the gate task's board id, written by the loader.
- Tasks are a numbered list under their milestone. A task line is `<n>. **<title>**`, then optional markers in this order:
  - `[p0]` to `[p3]` priority. Leave it out for the default, p2.
  - `[human]` for a task agents must not take.
  - `{<epic-slug>}` to override the milestone's epic.
  - `(waits: ...)` with keys of other tasks. `3` is task 3 of the same milestone, `M1.2` is task 2 of M1, and `M1` is the whole milestone.
  - `<!-- orch#12 -->` the board id, written by the loader. Don't write it by hand.
- Indented lines under a task are its description, in Markdown.
- Task keys (`M1.2`) are positions, so don't renumber tasks that are already on the board. Add new tasks at the end of a milestone's list.
