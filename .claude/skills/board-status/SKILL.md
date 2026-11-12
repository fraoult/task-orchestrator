---
name: board-status
description: Summarise the kanban board for this repo's project - per-milestone progress, what's ready for agents, what's blocked, what's in review, and what's waiting on the user. Use when asked for board status, progress, what's next, or what needs the user.
---

# Board status

Read only. Never change tasks here.

1. `list_tasks` for the repo's project with `status: all`.
2. Group by milestone tag (`m1`, `m2`, ...) in order, and for each give done out of total, ignoring the gate task (tagged `gate`).
3. Then list, each with id, title and one line of why:
   - **Needs you:** human tasks in todo that aren't waiting, gate tasks whose milestone tasks are all done (say "run the done-when test"), tasks tagged `approved` (ready to merge, with the PR link), and tasks tagged `stuck`.
   - **In review:** the rest of the review column, with PR links.
   - **In progress:** the doing column, with how long since it was last updated. Flag anything untouched for over an hour as possibly abandoned.
   - **Ready for agents:** unblocked, non-human todo tasks in board order, the first few only. This is what `work-task` will take next.
   - **Blocked:** count per milestone, and the tasks that unblock the most once done.

Keep it short: the user reads it to decide what to do next.
