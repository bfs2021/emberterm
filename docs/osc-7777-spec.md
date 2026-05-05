# OSC 7777 — Agent Integration Protocol for Terminal Emulators

**Specification version:** 0.1 (draft)
**Document status:** Draft — open for review
**License:** CC-BY-4.0
**Editor:** EmberTerm Project
**Last updated:** 2026-05-05

> A vendor-neutral protocol for AI coding agents and terminal emulators
> to communicate structured information — tool calls, diffs, approval
> requests, session marks — over the existing PTY channel.

---

## Table of Contents

1. [Abstract](#1-abstract)
2. [Motivation](#2-motivation)
3. [Conformance](#3-conformance)
4. [Wire Format](#4-wire-format)
5. [Common Payload Conventions](#5-common-payload-conventions)
6. [Capability Negotiation](#6-capability-negotiation)
7. [Verbs (Agent → Terminal)](#7-verbs-agent--terminal)
8. [Verbs (Terminal → Agent)](#8-verbs-terminal--agent)
9. [End-to-End Examples](#9-end-to-end-examples)
10. [Error Handling](#10-error-handling)
11. [Security Considerations](#11-security-considerations)
12. [Implementation Notes](#12-implementation-notes)
13. [Versioning & Future Compatibility](#13-versioning--future-compatibility)
14. [Acknowledgments](#14-acknowledgments)
15. [Changelog](#15-changelog)

---

## 1. Abstract

OSC 7777 defines a JSON-payload protocol layered on top of the standard
ANSI Operating System Command (OSC) escape-sequence mechanism. It allows
agents — particularly AI coding assistants such as Claude Code, Codex CLI,
OpenCode, and Aider — to communicate structured semantic events to a
terminal emulator that can render them as native UI affordances (tool-call
cards, inline diffs, approval prompts, session marks), while remaining
fully transparent to terminals that do not implement the protocol.

The protocol is bidirectional and runs entirely over the existing PTY
input/output channel. It requires no side-channel IPC, no authentication,
and no platform-specific support.

---

## 2. Motivation

Modern AI coding agents emit rich semantic events: "I am about to read a
file," "Here is a proposed patch," "Approve this destructive command?"
Today these events are squeezed into the same byte stream as ordinary
program output, rendered as ad-hoc TUIs (boxes, colors, cursor tricks)
that vary per agent and degrade poorly when terminal width or capability
varies.

Terminals, in turn, have no way to know which bytes are agent semantics
versus ordinary output. They cannot offer block-aware UX, native diff
viewers, or approval overlays without parsing each agent's bespoke output.

OSC 7777 provides a single shared vocabulary so:

- Agents emit semantic events once, in a standard format.
- Any conforming terminal renders them with native UI.
- Terminals that do not implement OSC 7777 ignore the sequences silently.
- New agents and new terminals interoperate without coordination.

---

## 3. Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY**
in this document are to be interpreted as described in
[RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) and
[RFC 8174](https://datatracker.ietf.org/doc/html/rfc8174).

### 3.1 Roles

This specification defines two roles:

- **Agent** — a program running inside a PTY that emits OSC 7777 sequences.
- **Terminal** — a terminal emulator that hosts the PTY and consumes OSC 7777
  sequences.

A single program **MAY** play both roles (e.g., a multiplexer relaying
sequences between an inner agent and an outer terminal).

### 3.2 Conformance Levels

A **Level 1 conforming terminal** **MUST**:

- Recognize the OSC 7777 wire format (Section 4) and silently consume any
  sequence it does not understand without writing it to the screen.
- Implement `agent.hello`, `agent.query`, and the `agent.reply` response.
- Return accurate capability information when queried.

A **Level 2 conforming terminal** **MUST** additionally:

- Implement at least one of: `tool.start` / `tool.progress` / `tool.end`,
  `diff`, or `approval`.
- Render implemented verbs as user-visible UI.
- Send `reply` messages back to the agent for any verb requiring user input.

A **conforming agent** **MUST**:

- Emit the wire format defined in Section 4 with valid JSON payloads.
- Send `agent.hello` once at startup before any other verb.
- Tolerate the terminal not implementing the protocol (no response to
  `agent.query`, no `reply` to actionable verbs) and degrade gracefully.

---

## 4. Wire Format

### 4.1 Sequence Structure

Every OSC 7777 message is an Operating System Command sequence with the
numeric identifier `7777`:

```
ESC ] 7777 ; <verb> ; <payload> ST
```

Where:

- `ESC` is the byte `0x1B`.
- `]` is the byte `0x5D`.
- `7777` is the literal four-byte ASCII string `"7777"`.
- `;` is the byte `0x3B`.
- `<verb>` is a dotted ASCII identifier (Section 4.3).
- `<payload>` is a UTF-8 encoded JSON object (Section 4.4).
- `ST` (String Terminator) **MUST** be either:
  - The two-byte sequence `ESC \` (`0x1B 0x5C`) — **preferred**, or
  - The single byte `BEL` (`0x07`) — **accepted for compatibility**.

Conforming terminals **MUST** accept both terminators. Conforming agents
**SHOULD** emit `ESC \`.

### 4.2 Encoding Rules

- Payload bytes **MUST** be valid UTF-8.
- The payload **MUST NOT** contain a literal `ESC` byte (`0x1B`) or `BEL`
  byte (`0x07`); these characters in JSON string values **MUST** be escaped
  (`\u001b`, `\u0007`).
- The payload **MUST NOT** contain a literal newline (`0x0A`); use `\n` JSON
  escape if needed.
- Maximum sequence length is implementation-defined. Conforming terminals
  **MUST** support payloads of at least 64 KiB. Agents **SHOULD** keep
  payloads under 16 KiB; for larger data (e.g., diffs of huge files) split
  into multiple verbs or pass a file reference.

### 4.3 Verb Names

Verbs are dotted ASCII identifiers matching the regex:

```
^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)*$
```

Verbs defined by this specification use the namespaces `agent.*`, `tool.*`,
`diff`, `approval`, `block.*`, and `session.*`.

Implementations **MAY** define experimental verbs in the `x.<vendor>.*`
namespace (e.g., `x.embertermo.foo`). Such verbs **MUST NOT** be required
for interoperability and other implementations **MUST** ignore them
silently.

### 4.4 Payload

The payload is a single JSON object. It **MUST**:

- Be a JSON object (not a JSON array, string, number, or null).
- Include the field `"v"` (integer) — the payload schema version. This
  specification defines version `1`.
- Include the field `"id"` (string) for verbs requiring a response. The
  `id` is opaque to the terminal but **MUST** be unique within a session
  for the lifetime of the request.

---

## 5. Common Payload Conventions

### 5.1 Required Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `v` | integer | Always | Payload schema version. Currently `1`. |
| `id` | string | When a reply is expected | Opaque request identifier. |

### 5.2 Optional Common Fields

| Field | Type | Description |
|---|---|---|
| `parent_id` | string | Identifier of a parent operation (e.g., parent tool call). |
| `meta` | object | Implementation-defined metadata. Terminals **MUST** preserve unknown keys when relaying. |

### 5.3 Identifiers

The `id` field is an opaque string. Agents **MAY** use UUIDs, monotonic
counters, or any other scheme. Terminals **MUST NOT** parse, mutate, or
infer meaning from `id` values; they **MUST** echo them verbatim in
matching replies.

`id` strings **MUST** be at most 128 ASCII characters and **MUST NOT**
contain control characters.

---

## 6. Capability Negotiation

### 6.1 Hello

The first OSC 7777 sequence an agent emits **MUST** be `agent.hello`:

```json
{
  "v": 1,
  "name": "<agent-name>",
  "version": "<agent-version>",
  "pid": <integer>
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Stable identifier (e.g., `"claude-code"`). |
| `version` | string | Yes | Agent version (any format). |
| `pid` | integer | No | OS process ID. |

The terminal **SHOULD** display the agent's identity in its UI (e.g.,
status bar) but **MUST NOT** treat the value as authoritative for
authorization decisions.

### 6.2 Query Capabilities

To learn which verbs the terminal implements, the agent emits:

```json
{ "v": 1, "id": "<id>", "ask": "capabilities" }
```

The terminal **MUST** reply (Section 8.1) with:

```json
{
  "v": 1,
  "id": "<id>",
  "result": {
    "osc_7777": ["tool.*", "diff", "approval", "block.annotate", "session.mark"],
    "level": 2,
    "blocks": true,
    "graphics": ["kitty"],
    "terminal": { "name": "ember", "version": "0.1.0" }
  }
}
```

| Field | Type | Description |
|---|---|---|
| `osc_7777` | array of strings | Implemented verbs (`*` wildcard allowed). |
| `level` | integer | Conformance level (1 or 2). |
| `blocks` | boolean | Terminal supports OSC 133 block marks. |
| `graphics` | array of strings | Image protocols supported (`"kitty"`, `"sixel"`, `"iterm2"`). |
| `terminal` | object | Terminal name and version. |

If the terminal does not implement OSC 7777, no reply will arrive. Agents
**MUST** time out after a reasonable interval (recommended: 250 ms) and
treat absence of reply as "no OSC 7777 support."

### 6.3 Graceful Degradation

If the terminal does not advertise a verb in `capabilities.osc_7777`, the
agent **SHOULD NOT** emit that verb and **SHOULD** fall back to plain-text
or in-band TUI rendering.

---

## 7. Verbs (Agent → Terminal)

### 7.1 `tool.start`

Notify the terminal that the agent is beginning a tool call.

```json
{
  "v": 1,
  "id": "t_42",
  "name": "edit_file",
  "summary": "Edit src/main.rs",
  "args": { "path": "src/main.rs", "lines": [10, 25] },
  "icon": "pencil",
  "parent_id": "t_40"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | Yes | Tool call identifier. |
| `name` | string | Yes | Tool identifier (e.g., `"edit_file"`). |
| `summary` | string | Yes | Short human-readable description (≤ 80 chars). |
| `args` | object | No | Tool arguments (free-form). |
| `icon` | string | No | Icon hint (`"pencil"`, `"file"`, `"terminal"`, `"network"`, `"search"`, `"bug"`). |
| `parent_id` | string | No | Identifier of a parent tool call. |

The terminal **SHOULD** render an inline tool card associated with the
current block. The card **SHOULD** display the tool name, summary, and a
running indicator until `tool.end` arrives.

### 7.2 `tool.progress`

Update an in-flight tool call.

```json
{
  "v": 1,
  "id": "t_42",
  "pct": 60,
  "message": "Applying patch...",
  "phase": "writing"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | Yes | Matches a prior `tool.start`. |
| `pct` | integer | No | Progress percentage 0–100. |
| `message` | string | No | Status message (≤ 120 chars). |
| `phase` | string | No | Implementation-defined phase identifier. |

The terminal **MAY** ignore progress updates if it does not render
progress UI. Updates with no matching `tool.start` **MUST** be silently
dropped.

### 7.3 `tool.end`

Mark a tool call as complete.

```json
{
  "v": 1,
  "id": "t_42",
  "status": "ok",
  "summary": "Edited 12 lines",
  "duration_ms": 1200,
  "error": null
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | Yes | Matches a prior `tool.start`. |
| `status` | string | Yes | One of `"ok"`, `"error"`, `"cancelled"`. |
| `summary` | string | No | Human-readable result (≤ 120 chars). |
| `duration_ms` | integer | No | Total elapsed milliseconds. |
| `error` | object \| null | No | Error details when `status` is `"error"`. Shape: `{ "code": string, "message": string }`. |

The terminal **SHOULD** collapse the tool card to a one-line summary
showing the final status.

### 7.4 `diff`

Request that the terminal render a unified diff with action affordances.

```json
{
  "v": 1,
  "id": "d_1",
  "path": "src/main.rs",
  "patch": "--- a/src/main.rs\n+++ b/src/main.rs\n@@ ...",
  "language": "rust",
  "actions": ["accept", "reject", "edit"]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | Yes | Diff identifier. |
| `path` | string | Yes | Target file path (display only). |
| `patch` | string | Yes | Unified diff text. |
| `language` | string | No | Language hint for syntax highlighting. |
| `actions` | array of strings | Yes | Subset of `"accept"`, `"reject"`, `"edit"`. |

The terminal **SHOULD** render an inline patch viewer with action buttons.
On user action, the terminal **MUST** send a `reply` (Section 8.1) with
`result` set to the chosen action string, or `null` if the user dismissed
without choosing.

For multi-file changes, agents **SHOULD** emit one `diff` verb per file.
A future revision of this spec **MAY** introduce a `diff.batch` verb.

### 7.5 `approval`

Request explicit user approval for a destructive or sensitive operation.

```json
{
  "v": 1,
  "id": "a_1",
  "title": "Run shell command",
  "detail": "rm -rf node_modules",
  "risk": "destructive",
  "actions": ["allow_once", "allow_session", "deny"],
  "default": "deny"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | Yes | Approval identifier. |
| `title` | string | Yes | Short title (≤ 60 chars). |
| `detail` | string | Yes | Multi-line detail (e.g., the command). |
| `risk` | string | Yes | One of `"safe"`, `"caution"`, `"destructive"`. |
| `actions` | array of strings | Yes | Subset of `"allow_once"`, `"allow_session"`, `"allow_always"`, `"deny"`. |
| `default` | string | No | Default highlighted action (recommended: `"deny"` for destructive). |

The terminal **SHOULD** render a prominent UI overlay (e.g., a sheet or
modal) and **MUST** send a `reply` with `result` set to the chosen action
string. The terminal **MAY** include a timeout; if so, the timeout
**SHOULD** default to the safest action.

> **Note:** OSC 7777 approvals are a UX layer, not a security boundary.
> See Section 11.

### 7.6 `block.annotate`

Annotate the current shell block (the most recently started OSC 133 block)
with metadata.

```json
{
  "v": 1,
  "tags": ["agent", "tool-call"],
  "agent_id": "claude-code",
  "fold": "auto",
  "title": "Refactor main.rs"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `tags` | array of strings | No | Free-form tags (≤ 16 entries, ≤ 32 chars each). |
| `agent_id` | string | No | Stable agent identifier. |
| `fold` | string | No | One of `"auto"`, `"open"`, `"closed"`. |
| `title` | string | No | Override for block title (≤ 80 chars). |

If no current block exists (no `OSC 133 ; A` mark seen), the terminal
**MUST** silently drop the annotation.

### 7.7 `session.mark`

Insert a labeled marker in the session timeline (e.g., a checkpoint after
a logical unit of work).

```json
{
  "v": 1,
  "label": "Refactor complete",
  "kind": "checkpoint"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `label` | string | Yes | Marker label (≤ 80 chars). |
| `kind` | string | No | One of `"checkpoint"`, `"phase"`, `"warning"`, `"error"`. |

The terminal **SHOULD** render a visible marker in the session view
(e.g., a horizontal divider or a navigable bookmark in the scrollback).

### 7.8 `agent.bye`

Notify the terminal that the agent is exiting cleanly. This verb is
optional; terminals **MUST NOT** require it.

```json
{ "v": 1, "reason": "completed" }
```

| Field | Type | Required | Description |
|---|---|---|---|
| `reason` | string | No | Free-form exit reason. |

---

## 8. Verbs (Terminal → Agent)

The terminal communicates back to the agent by emitting OSC 7777 sequences
on the **PTY input stream** — the side that the agent reads from as its
standard input. This is the same mechanism used for traditional terminal
query responses (e.g., DSR, DA1).

### 8.1 `reply`

Response to an agent-initiated request that requires user input.

```
ESC ] 7777 ; reply ; { "v":1, "id":"d_1", "result":"accept" } ESC \
```

```json
{
  "v": 1,
  "id": "<matches request id>",
  "result": <varies by request>,
  "error": null
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | Yes | Matches the corresponding agent request. |
| `result` | varies | Yes | Result value (varies by verb; `null` if user dismissed). |
| `error` | object \| null | No | Error details if the terminal could not honor the request. |

`result` types by originating verb:

| Verb | `result` type | Possible values |
|---|---|---|
| `agent.query` (capabilities) | object | See Section 6.2 |
| `diff` | string \| null | `"accept"`, `"reject"`, `"edit"`, or `null` |
| `approval` | string \| null | One of declared `actions`, or `null` |

### 8.2 `event`

Terminal-initiated events that did not originate from a specific agent
request. Used for user actions that target a specific agent operation
(e.g., a Cancel button on a tool card).

```
ESC ] 7777 ; event ; { "v":1, "kind":"tool.cancel", "id":"t_42" } ESC \
```

| Field | Type | Required | Description |
|---|---|---|---|
| `kind` | string | Yes | Event identifier. |
| `id` | string | Sometimes | Identifier of the targeted operation. |

Defined event kinds:

| `kind` | Description | Required fields |
|---|---|---|
| `tool.cancel` | User requested cancellation of a tool call | `id` (matching `tool.start`) |
| `diff.dismiss` | User dismissed a pending diff without choosing | `id` |
| `approval.timeout` | An approval prompt timed out | `id`, `default_action` |
| `agent.interrupt` | User pressed an interrupt key in a context targeting the agent | — |

Agents **SHOULD** handle `tool.cancel` by aborting the operation cleanly
and emitting `tool.end` with `status: "cancelled"`. Agents **MAY** ignore
events they do not understand.

---

## 9. End-to-End Examples

### 9.1 Capability Discovery

```
agent → terminal:
  ESC ] 7777 ; agent.hello ; {"v":1,"name":"claude-code","version":"1.0.0","pid":12345} ESC \

agent → terminal:
  ESC ] 7777 ; agent.query ; {"v":1,"id":"q1","ask":"capabilities"} ESC \

terminal → agent (via PTY input):
  ESC ] 7777 ; reply ; {"v":1,"id":"q1","result":{"osc_7777":["tool.*","diff","approval"],"level":2,"blocks":true,"terminal":{"name":"ember","version":"0.1.0"}}} ESC \
```

### 9.2 Tool Call with Progress

```
agent → terminal:
  ESC ] 7777 ; tool.start ; {"v":1,"id":"t_1","name":"read_file","summary":"Read src/main.rs","args":{"path":"src/main.rs"}} ESC \

agent → terminal:
  ESC ] 7777 ; tool.progress ; {"v":1,"id":"t_1","pct":50,"message":"Streaming..."} ESC \

agent → terminal:
  ESC ] 7777 ; tool.end ; {"v":1,"id":"t_1","status":"ok","summary":"Read 412 lines","duration_ms":80} ESC \
```

### 9.3 Diff with User Action

```
agent → terminal:
  ESC ] 7777 ; diff ; {"v":1,"id":"d_1","path":"src/main.rs","patch":"--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    init_logging();\n     run();\n }\n","language":"rust","actions":["accept","reject","edit"]} ESC \

[user clicks Accept]

terminal → agent:
  ESC ] 7777 ; reply ; {"v":1,"id":"d_1","result":"accept"} ESC \
```

### 9.4 Approval with Cancellation

```
agent → terminal:
  ESC ] 7777 ; approval ; {"v":1,"id":"a_1","title":"Run command","detail":"rm -rf dist/","risk":"destructive","actions":["allow_once","deny"],"default":"deny"} ESC \

[user presses Y]

terminal → agent:
  ESC ] 7777 ; reply ; {"v":1,"id":"a_1","result":"allow_once"} ESC \
```

### 9.5 User Cancels a Running Tool

```
agent → terminal:
  ESC ] 7777 ; tool.start ; {"v":1,"id":"t_5","name":"build","summary":"Run cargo build"} ESC \

[user clicks Cancel on the tool card]

terminal → agent:
  ESC ] 7777 ; event ; {"v":1,"kind":"tool.cancel","id":"t_5"} ESC \

[agent aborts the build]

agent → terminal:
  ESC ] 7777 ; tool.end ; {"v":1,"id":"t_5","status":"cancelled","summary":"Build cancelled by user"} ESC \
```

---

## 10. Error Handling

### 10.1 Malformed Sequences

A conforming terminal **MUST** silently discard sequences with:

- An invalid wire format (missing fields, unterminated sequence, etc.)
- A malformed JSON payload
- A payload that is not a JSON object
- An unknown verb
- A known verb with missing required fields
- A known verb with type-mismatched fields

Silent discard means the sequence **MUST NOT** be written to the screen,
**MUST NOT** generate visible error output, and **MUST NOT** terminate
the connection. Terminals **MAY** log discarded sequences for debugging.

### 10.2 Stale Identifiers

For verbs referencing a prior `id` (e.g., `tool.progress`, `tool.end`),
if no matching prior verb exists, the terminal **MUST** silently drop the
sequence.

### 10.3 Reply Timeouts

For verbs requiring a `reply` (`agent.query`, `diff`, `approval`), the
agent **MUST** be prepared for the reply to never arrive (e.g., the user
walks away). Agents **SHOULD** implement timeouts appropriate to the
operation:

| Verb | Recommended timeout |
|---|---|
| `agent.query` | 250 ms |
| `diff` | None (wait indefinitely) |
| `approval` | None or implementation-defined; never auto-approve on timeout |

### 10.4 Replies for Unknown Requests

If the terminal sends a `reply` with an `id` the agent does not recognize
(e.g., the agent forgot the request, or the request was canceled), the
agent **MUST** silently discard the reply.

---

## 11. Security Considerations

### 11.1 OSC 7777 Is Not a Security Boundary

OSC 7777 is a **display and UX protocol**, not an authentication or
authorization mechanism. Anything running inside the agent's PTY can emit
OSC 7777 sequences — including programs the agent runs and content the
agent renders verbatim from untrusted sources (web pages, files, model
output).

In particular:

- The `approval` verb **MUST NOT** be the sole defense against destructive
  operations. Agents **MUST** independently verify dangerous actions.
- Terminals **MUST NOT** treat the `agent.hello` `name` field as
  authoritative; any program can claim to be any agent.
- Terminals **MUST NOT** grant elevated privileges to programs based on
  OSC 7777 capabilities.

### 11.2 Injection via Untrusted Output

A program running in an agent's PTY (or a file the agent reads and prints)
could contain arbitrary OSC 7777 sequences. A terminal that displays
agent output verbatim could be tricked into rendering unauthorized
approval prompts, fake diffs, or misleading session marks.

Mitigations:

- Agents **SHOULD** sanitize untrusted bytes before printing them by
  stripping or escaping `ESC` characters.
- Terminals **MAY** display a visual indicator (e.g., a small icon) on
  every OSC 7777-rendered UI element identifying which agent emitted it.
- Terminals **SHOULD NOT** auto-confirm any approval based on OSC 7777
  alone.

### 11.3 Privacy

Payloads are transmitted over the PTY; on most systems this is local IPC
and not network-exposed. However:

- `tool.start.args` may contain sensitive data (file paths, secrets).
- `diff.patch` may contain proprietary code.
- Terminals **MUST NOT** include OSC 7777 payload contents in telemetry,
  crash reports, or logs sent off-machine without explicit user consent.

### 11.4 Resource Exhaustion

A malicious or buggy agent could emit a flood of OSC 7777 sequences.
Conforming terminals **SHOULD** rate-limit OSC 7777 sequence processing
and **MAY** discard sequences when limits are exceeded. Recommended limit:
1000 sequences per second per session.

---

## 12. Implementation Notes

### 12.1 For Agent Authors

- Emit `agent.hello` immediately on startup, before any other terminal
  output. This lets the terminal label your session and prepare any UI.
- Always emit `agent.query` and check the result before using verbs
  beyond Level 1. Cache the capabilities for the session.
- Use opaque, monotonically-unique IDs (UUIDs work well). Do not embed
  meaning in IDs; the terminal will not parse them.
- Detect non-TTY stdout (e.g., piped to a file) and skip emitting OSC
  7777 entirely. The protocol is meaningless to a file.
- When falling back due to no terminal support, render plain-text or
  in-band TUI as you would today. OSC 7777 is purely additive.

### 12.2 For Terminal Authors

- Hook your existing OSC parser. The wire format is identical to other
  OSC sequences; only the dispatch on the numeric identifier `7777` is
  new.
- Treat the `agent.hello` event as an opportunity to update UI chrome
  (e.g., status bar) but make no behavioral changes that affect security.
- The `block.annotate` verb requires OSC 133 prompt-mark support to be
  meaningful. If you do not implement OSC 133, return `blocks: false` in
  capabilities and silently drop `block.annotate`.
- For the reply channel, write the OSC 7777 reply sequence to the PTY
  master's input side. On both ConPTY and Unix PTYs this is a single
  `write()` call; no platform-specific code required.

### 12.3 Detecting OSC 7777 Support Without Querying

Agents that cannot afford a 250 ms startup delay **MAY** check the
environment variable `TERM_OSC_7777=1` set by terminals that implement
this specification. This is an optional optimization; terminals **MAY**
set this variable, and agents **MAY** read it, but absence does not imply
non-support — a `agent.query` reply is still authoritative.

---

## 13. Versioning & Future Compatibility

### 13.1 Protocol Version

The `v` field in every payload identifies the protocol schema version.
This specification defines version `1`.

Future versions **MAY**:

- Add new verbs (existing implementations ignore them silently).
- Add new optional fields (existing implementations ignore them silently).
- Add new event kinds (existing implementations ignore them silently).

Future versions **MUST NOT**:

- Remove or rename existing verbs or fields without a major version bump.
- Change the wire format of OSC 7777 itself.
- Change the meaning of existing field values.

### 13.2 Major Version Changes

If incompatible changes become necessary, a new OSC numeric identifier
(e.g., 7778) **SHOULD** be allocated rather than overloading 7777 with
breaking changes. This guarantees forward-compatible coexistence.

### 13.3 Vendor Extensions

Implementations **MAY** define experimental verbs in the `x.<vendor>.*`
namespace. Such verbs **MUST NOT** be required for interoperability and
**MUST** be safe for other implementations to ignore.

Example: `x.embertermo.confetti` for celebratory animations.

---

## 14. Acknowledgments

This specification draws inspiration from:

- **OSC 8** (terminal hyperlinks) — for the convention of carrying
  semantic data over OSC sequences.
- **OSC 133** (shell integration) — for the prompt-mark model that
  enables block-aware UX.
- **iTerm2's OSC 1337** — for proving rich vendor extensions over OSC
  are practical.
- **Kitty graphics protocol** — for the convention of structured payloads
  in OSC sequences.
- **Warp's block UX** — for demonstrating the value of agent-aware
  terminal UI.
- **Model Context Protocol (MCP)** — for the broader pattern of
  structured AI-tool integration; OSC 7777 is the *display layer*
  complementary to MCP's *data layer*.

---

## 15. Changelog

### 0.1 (2026-05-05) — Initial draft

- First public draft.
- Defines wire format, conformance levels, capability negotiation.
- Defines verbs: `agent.hello`, `agent.query`, `agent.bye`, `tool.start`,
  `tool.progress`, `tool.end`, `diff`, `approval`, `block.annotate`,
  `session.mark`.
- Defines reply channel and `reply` / `event` verbs.
- Open issues for v0.2:
  - `diff.batch` for atomic multi-file changes
  - Streaming `tool.output` verb for incremental tool output rendering
  - `agent.thinking` verb to render reasoning blocks distinctly

---

*This specification is published under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/).*
*Comments, corrections, and extension proposals are welcome via pull request.*