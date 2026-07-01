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
  assert.match(tray, /refresh_menu/);
  assert.match(tray, /list_profiles_with_runtime_state/);
  assert.match(tray, /start_profile_by_id/);
  assert.match(tray, /toggle_proxy_by_name/);
  assert.doesNotMatch(tray, /block_on/);

  assert.match(commands, /sync_profile_state\(&app\)/);
});
