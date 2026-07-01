import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("status bar menu exposes profile and proxy quick actions", async () => {
  const tray = await readFile(new URL("../src-tauri/src/tray.rs", import.meta.url), "utf8");
  const commands = await readFile(new URL("../src-tauri/src/commands.rs", import.meta.url), "utf8");

  assert.match(tray, /Check FRP Manager Update/);
  assert.doesNotMatch(tray, /Check frpc Update/);
  assert.match(tray, /Profiles/);
  assert.match(tray, /Start/);
  assert.match(tray, /Stop/);
  assert.match(tray, /toggle-proxy:/);
  assert.match(tray, /start-profile:/);
  assert.match(tray, /stop-profile:/);
  assert.match(tray, /"quit" => quit_app\(app, 0\)/);
  assert.doesNotMatch(tray, /"quit" => app\.exit\(0\)/);
  assert.match(tray, /pub fn quit_app/);
  assert.match(tray, /stop_all\(\)\.await/);
  assert.match(tray, /refresh_menu/);
  assert.match(tray, /list_profiles_with_runtime_state/);
  assert.match(tray, /start_profile_by_id/);
  assert.match(tray, /toggle_proxy_by_name/);
  assert.match(tray, /pub async fn refresh_menu/);
  assert.match(tray, /pub async fn sync_profile_state/);
  assert.match(tray, /state\s*\.registry\s*\.write\(\)\s*\.await/);
  assert.doesNotMatch(tray, /block_on/);
  assert.doesNotMatch(tray, /try_write/);
  assert.doesNotMatch(tray, /\.or_else\(\|\| store\.list\(\)\.ok\(\)\)/);

  assert.match(commands, /sync_profile_state\(&app\)\.await/);
});
