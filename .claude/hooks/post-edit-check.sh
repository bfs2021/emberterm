#!/usr/bin/env bash
# Post-edit hook: run a fast type-check on the modified file.
# Runs after every Edit or Write tool use.
# Exit 0 always — this is informational, never blocking.

# Parse file_path from the tool input JSON
FILE=$(node -e "
  try {
    const d = JSON.parse(process.env.CLAUDE_TOOL_INPUT || '{}');
    process.stdout.write(d.file_path || '');
  } catch(e) {
    process.stdout.write('');
  }
" 2>/dev/null)

if [[ -z "$FILE" ]]; then
  exit 0
fi

# Normalise to forward slashes for matching
FILE_NORM="${FILE//\\//}"

if [[ "$FILE_NORM" == *.rs ]]; then
  echo "┌─ cargo check ─────────────────────────────"
  cargo check --manifest-path src-tauri/Cargo.toml 2>&1 \
    | grep -E "^error|^warning\[|^  -->" \
    | head -30
  echo "└──────────────────────────────────────────"

elif [[ "$FILE_NORM" == *.svelte ]] || [[ "$FILE_NORM" == *.ts ]]; then
  echo "┌─ svelte-check ────────────────────────────"
  (cd frontend && npm run check 2>&1) \
    | grep -E "Error|Warning|error|warning" \
    | grep -v "^>" \
    | head -30
  echo "└──────────────────────────────────────────"
fi

exit 0
