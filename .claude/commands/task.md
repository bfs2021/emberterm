# /task — Look up or update tasks in TASKS.md

Interact with the task list in docs/TASKS.md.

If $ARGUMENTS is empty:
  - Read docs/TASKS.md
  - List all unchecked `[ ]` tasks in Phase 1, grouped by section
  - Show each as: `[Phase] Section › Task description`

If $ARGUMENTS starts with "done ":
  - The rest of the argument is a substring of a task description
  - Find the matching unchecked task in docs/TASKS.md
  - Mark it as done: change `[ ]` to `[x]`
  - Confirm which line was updated

If $ARGUMENTS is any other text:
  - Search docs/TASKS.md for unchecked tasks matching the text (case-insensitive)
  - Show up to 10 matching tasks with their phase and section context

Examples:
  /task                        → show all open Phase 1 tasks
  /task profile                → find tasks mentioning "profile"
  /task done drag to reorder   → mark the drag-reorder tab task as done
