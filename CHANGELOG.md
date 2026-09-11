# Changelog

## 0.0.9 — 2026-09-11

### Highlights
- **Simplified Chinese (zh-CN) localization** for the whole app: side panel, workbench, profile and proxy editors, table headers, the runtime and update dialogs, and the tray menu. A language switcher lives next to the "add profile" button in the sidebar; the active language persists across launches.
- **Log rotation** for each profile's `frpc` log so long-running tunnels no longer grow without bound. 2 MB per file, ten files kept on disk; the viewer only reads the tail.
- **Auto-start on launch** for individual profiles: a small badge in the sidebar marks a profile as auto-started, and the workbench exposes it on the profile's context menu (set up by the right-click menu, with a "已开启/已关闭" state column so all rows share a left edge).
- **Proxy hot reload** without restarting frpc: editing a proxy or its server/port now goes through the profile's admin API (`/api/reload`) first and only restarts the process as a fallback.

### Bug fixes and polish
- Stopped the profile context menu from wrapping its label on the "Start on launch" row.
- Reworked the same menu to spell out the auto-start state ("已开启" / "已关闭") instead of reserving a checkbox column that indented every other item.
- The workbench header no longer leaks English field names — it now reads `Server {addr}:{port} · {auth}` (English) or `服务端 {addr}:{port} · {auth}` (Chinese).
- Localized the four OIDC labels in the profile editor (client ID / client secret / audience / token endpoint URL).
- Fixed a malformed inline `[webServer]` table in generated `profile.toml` that previous versions of the profile reader could not parse.

### Internal
- Added `log_store` and `admin_api` modules in the Rust side, with tests for rotation boundaries and admin URL resolution.
- Added `settings` module so the language choice survives restarts (it is stored in `~/Library/Application Support/com.local.frpmanager/settings.json` on macOS).
- New i18n test suite that asserts key parity, placeholder parity, and a low untranslated ratio across locales.
