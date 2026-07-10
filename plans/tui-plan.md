# TUI for Pi reload and reset

## Goal

Add a keyboard-driven terminal UI to the `herdr-pi-reloader` plugin. The existing Herdr shortcut should open the TUI in a managed overlay pane. From there, the user can reload or reset all eligible Pi instances and inspect the final summary before closing the overlay.

This document is an implementation plan only. Do not broaden the scope beyond the decisions recorded below.

## Current codebase

- The project is a Rust binary using Tokio.
- `src/main.rs` exposes the `reload` and `reset` CLI commands.
- `src/herdr.rs` retrieves the agent list and runs commands in Herdr panes.
- `src/pi.rs` contains reload/reset selection and execution logic.
- Reload currently runs sequentially and sends `/reload` to eligible Pi panes.
- Reset currently runs candidates concurrently. Each reset sends `/quit`, waits up to 15 seconds for the old Pi instance to disappear, then runs `pi --session <escaped-path>` in the pane.
- Both operations currently accept only Pi agents whose status is `idle` or `done`.
- `herdr-plugin.toml` currently exposes a single `reload` action that directly invokes the CLI reload command.
- The local Herdr config currently binds `prefix+alt+r` to `herdr-pi-reloader.pi-reloader.reload`.
- Herdr 0.7.3 supports manifest-declared managed terminal panes. The default `overlay` placement creates the desired temporary overlay.

## User flow

1. The user presses `prefix+alt+r` in Herdr.
2. Herdr invokes the plugin's `open` action.
3. The action opens the plugin's manifest-declared TUI pane using `placement = "overlay"` and focuses it.
4. The initial menu displays, in this order:
   - `Reload all Pi` (selected by default)
   - `Reset all Pi`
5. The user changes the selection with Up/Down or `k`/`j`. Navigation wraps at both ends.
6. Enter immediately starts the selected operation. There is no confirmation screen for reset.
7. Only after Enter, the TUI retrieves the latest agent list from Herdr.
8. While the operation runs, the UI displays a spinner and one of:
   - `Reloading Pi instances...`
   - `Resetting Pi instances...`
9. The running operation cannot be cancelled from the TUI. Exit keys are ignored in this state.
10. After completion, the TUI displays the appropriate result heading and counters.
11. Enter, `q`, Escape, or Ctrl+C exits the result screen and terminates the TUI process. Herdr then removes the temporary overlay.

From the initial menu, `q`, Escape, and Ctrl+C exit without running an operation.

## Operation semantics

### Eligible Pi instances

An operation is performed only on agents that:

- are identified as `pi`; and
- have status `idle` or `done`; and
- contain all data required by the selected operation.

Agents other than Pi are ignored completely and must not affect any visible counter.

### Counters

The result screen shows:

- `Reloaded` for a reload operation, or `Reset` for a reset operation;
- `Skipped`;
- `Failed`.

Counter meanings:

- `Reloaded` / `Reset`: number of Pi instances for which the operation completed according to the existing success definition.
- `Skipped`: only Pi instances deliberately not touched because their status was neither `idle` nor `done`. This corresponds to the existing `skipped_unsafe_status` concept.
- `Failed`: known Pi instances that could not be operated on because required data was invalid or missing, plus runtime command failures.

If malformed agent data does not establish that the record represents Pi, ignore the record. If the record is known to represent Pi but lacks data such as pane ID, status, or the session information required for reset, count it as failed and include a short error message.

### Reload

- Preserve sequential execution.
- Do not parallelize it in this change.
- A reload succeeds when sending `/reload` through Herdr succeeds, matching current behavior.

### Reset

- Preserve concurrent execution across reset candidates.
- Preserve the existing 15-second wait for the old Pi instance to disappear.
- A reset succeeds after the old Pi exits and Herdr accepts the `pi --session ...` start command.
- Do not add a second wait that verifies the newly started Pi is detected.

### Overall result state

- `Success`: at least one operation succeeded and `Failed` is zero. Skipped instances do not prevent success.
- `Completed with errors`: `Failed` is greater than zero, including partial success.
- `Nothing to do`: successful operations and failures are both zero. This includes both no detected Pi instances and the case where every detected Pi was skipped. Always show the counters, so the two cases remain distinguishable.
- `Error`: the operation could not begin or complete at the global level, for example because the agent list could not be retrieved or parsed.

An `Error` screen must remain visible until the user explicitly closes it with Enter, `q`, Escape, or Ctrl+C.

## Error presentation

- Show full aggregate counts even when errors occurred.
- Show at most five short error messages.
- If more than five errors exist, append `...and N more errors`.
- Do not add scrolling, pagination, or another interaction mode for errors.
- Ensure errors are converted into UI data rather than printed while the TUI owns the terminal.

## TUI interaction and presentation

- Add `ratatui` and `crossterm` dependencies.
- Use English for all UI copy.
- Use the terminal alternate screen and raw mode, and restore terminal state on every normal error/exit path.
- Keep the layout simple and readable in Herdr's overlay.
- Use terminal-friendly colors without setting a custom background:
  - selected item: accent/cyan;
  - success: green;
  - warning/skipped: yellow;
  - errors: red.
- The UI must remain understandable without color.
- Handle terminal resize without crashing.
- Do not implement mouse support.
- Do not add a reset confirmation screen.
- Do not allow rerunning an action from the result screen; the result screen only closes.

### Keys

Menu state:

- Up or `k`: select previous item, wrapping around.
- Down or `j`: select next item, wrapping around.
- Enter: start the selected operation.
- `q`, Escape, Ctrl+C: exit.

Running state:

- All exit and selection keys are ignored. In particular, Ctrl+C must not interrupt an in-progress reset or reload.

Result and error states:

- Enter, `q`, Escape, Ctrl+C: exit.

## Suggested internal design

Keep terminal rendering separate from Pi operations and summary aggregation. A small explicit application state machine is appropriate:

- `Menu { selection }`
- `Running { operation, spinner_frame }`
- `Result { operation, summary }`
- `Error { message }`

Run the selected async operation independently from input/render ticks, then deliver its summary back to the UI event loop. This allows the spinner and resize handling to remain responsive while work is in progress. Do not permit the UI loop to cancel the operation.

Refactor operation return values as needed so the TUI receives structured, public summary data instead of relying on `Debug` output or `println!`. Preserve useful CLI output for direct `reload` and `reset` invocation, but prevent operation-level logging from corrupting the TUI screen.

The reset path needs an aggregate result comparable to reload's summary. It must count successful reset tasks, unsafe-status skips, known-Pi data failures, and runtime failures, while retaining up to all errors internally so the UI can apply its display limit.

## CLI and plugin integration

Preserve these direct CLI commands:

- `cargo run --quiet -- reload`
- `cargo run --quiet -- reset`

Add a dedicated CLI entrypoint for the TUI as required by the manifest pane command. The exact internal subcommand name may be `tui`.

Update `herdr-plugin.toml` so that:

- the old manifest action `reload` is replaced by one user-facing action with local ID `open`;
- the action opens the managed plugin pane rather than performing an operation itself;
- a `[[panes]]` entry declares the TUI command;
- the pane placement is explicitly `overlay`;
- the action and pane continue to support Linux and macOS through the plugin's existing platform declaration.

Use Herdr-provided environment variables, especially `HERDR_BIN_PATH` and `HERDR_PLUGIN_ID`, when opening the pane. Do not hard-code `/home/anrunt/.local/bin/herdr` inside the plugin.

Update the local file `~/.config/herdr/config.toml` so the existing keybinding remains `prefix+alt+r` but its command becomes:

```toml
command = "herdr-pi-reloader.pi-reloader.open"
```

Do not add a second shortcut. Do not retain the old manifest action merely for shortcut compatibility; the direct CLI reload command is the compatibility path.

## Explicit non-goals

- No tests in this change, including unit, snapshot, or live-Herdr integration tests.
- No mouse interaction.
- No confirmation before reset.
- No cancellation during an operation.
- No live per-pane progress list or live counters.
- No error-list scrolling.
- No parallel reload implementation.
- No verification that a reset Pi has been detected again after its start command.
- No singleton or duplicate-overlay protection. Assume the user invokes the shortcut once.
- No localization beyond English.
- No unrelated cleanup or broad architectural rewrite.

## Verification

Even though no tests are being added, the implementing agent should perform proportionate verification:

1. Run formatting and compile checks for the Rust project.
2. Confirm the plugin manifest remains valid and can be read by `herdr plugin list --plugin herdr-pi-reloader.pi-reloader --json` after Herdr reloads/re-reads it.
3. Confirm `prefix+alt+r` opens a focused overlay containing the TUI.
4. Verify Up/Down and `j`/`k` wrap and Enter starts the selected operation.
5. Verify `q`, Escape, and Ctrl+C close the menu without performing work.
6. Verify the running screen remains responsive and does not allow cancellation.
7. Verify reload and reset summaries classify succeeded, unsafe-status skipped, and failed Pi instances correctly while ignoring non-Pi agents.
8. Verify `Success`, `Completed with errors`, `Nothing to do`, and global `Error` states.
9. Verify no more than five error messages are displayed.
10. Verify Enter, `q`, Escape, and Ctrl+C close result/error screens and that the terminal is restored cleanly.
11. Verify the existing direct CLI `reload` and `reset` commands still work.

## Acceptance criteria

- The existing Herdr shortcut opens the plugin TUI in an overlay instead of immediately reloading Pi instances.
- The user can choose reload or reset entirely with the agreed keyboard controls.
- The selected action runs without an extra confirmation screen.
- Only eligible `idle`/`done` Pi instances are acted on.
- Final counts and headings follow the exact semantics in this plan.
- The result remains visible until explicitly dismissed.
- Exiting restores the terminal and closes the managed overlay.
- Direct CLI reload/reset behavior remains available.
- None of the explicit non-goals are introduced.
