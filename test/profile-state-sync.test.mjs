import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("profile state changes are synchronized between tray and main window", async () => {
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const api = await readFile(new URL("../src/lib/api.ts", import.meta.url), "utf8");
  const tray = await readFile(new URL("../src-tauri/src/tray.rs", import.meta.url), "utf8");
  const commands = await readFile(new URL("../src-tauri/src/commands.rs", import.meta.url), "utf8");

  assert.match(api, /PROFILE_STATE_CHANGED_EVENT/);
  assert.match(api, /listenProfileStateChanged/);
  assert.match(api, /@tauri-apps\/api\/event/);
  assert.match(app, /listenProfileStateChanged/);
  assert.match(app, /refreshProfiles\(selectedIdRef\.current,\s*true\)/);
  assert.match(tray, /PROFILE_STATE_CHANGED_EVENT/);
  assert.match(tray, /emit\(PROFILE_STATE_CHANGED_EVENT/);
  assert.match(tray, /sync_profile_state\(&app\)/);
  assert.match(commands, /sync_profile_state\(&app\)/);
  assert.doesNotMatch(commands, /refresh_tray_menu\(&app\)/);
});
