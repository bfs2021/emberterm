# /ui-review — Review a component against the design mockup

Compare a frontend component's current implementation against the canonical design in `references/design.html`.

Usage: `/ui-review $ARGUMENTS`
  — $ARGUMENTS should be the component name or file path, e.g. `StatusBar` or `ToolCard`

Steps:
1. Read `docs/review-ui-ux.md` to understand already-documented gaps
2. Read the component file from `frontend/src/`
3. Use the design knowledge from `references/design.html` (described in docs/review-ui-ux.md)
4. Compare: dimensions, spacing, colors (CSS variable names), interaction states, missing elements
5. Output a focused gap list for this component only:
   - ✅ Matches design
   - ⚠️ Minor deviation (describe)
   - ❌ Missing or wrong (describe what the design shows)
6. Ask if you should implement any of the fixes

If $ARGUMENTS is empty, show the summary from docs/review-ui-ux.md and ask which component to review.
