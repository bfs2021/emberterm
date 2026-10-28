# Security policy

EmberTerm runs shells on your machine with your permissions, so security reports are taken seriously.

## Report a vulnerability

Please do **not** open a public issue. Use GitHub's private reporting instead: open the repository's **Security** tab and choose **Report a vulnerability**. Include what you found, how to reproduce it, and the version (Settings → About).

You can expect a first answer within a few days. EmberTerm is maintained by one person, so fixes are made on a best-effort basis.

## Scope

In scope: the update check and installer signature verification, the shell hook injection in `packages/desktop/src/terminal/shell_hooks.rs`, OSC sequence handling (folder, title, clipboard) in the app, the Tauri IPC commands, and the release workflow.

Not treated as vulnerabilities:

- A shell tab runs commands as the user who started EmberTerm. That is what a terminal does.
- Windows builds are not code-signed and macOS builds are ad-hoc signed and not notarized, so SmartScreen and Gatekeeper warn on first launch. Updates are verified against the public key in `tauri.conf.json`.

## Network use

EmberTerm has no account and no telemetry. The only request it makes on its own is to GitHub Releases, when you choose **Settings → About → Check for updates**.

## Notes for operators

- Keep the updater private key (`TAURI_SIGNING_PRIVATE_KEY`) out of the repository and out of issues. Installed copies cannot verify later updates without it.
- Never paste the contents of `config.toml`, `profiles/` or `session.json` into an issue without checking them for paths and secrets first.
