import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("window close switches to status-bar mode and dock reopen shows the window", async () => {
  const lib = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const tray = await readFile(new URL("../src-tauri/src/tray.rs", import.meta.url), "utf8");
  const processManager = await readFile(
    new URL("../src-tauri/src/process_manager.rs", import.meta.url),
    "utf8",
  );

  assert.match(lib, /WindowEvent::CloseRequested/);
  assert.match(lib, /ActivationPolicy::Accessory/);
  assert.match(lib, /RunEvent::Reopen/);
  assert.match(lib, /show_main_window\(app/);
  assert.match(lib, /RunEvent::ExitRequested/);
  assert.match(lib, /api\.prevent_exit\(\)/);
  assert.match(lib, /quit_app\(app,\s*code\.unwrap_or\(0\)\)/);
  assert.match(tray, /pub fn show_main_window/);
  assert.match(tray, /pub fn quit_app/);
  assert.match(tray, /ActivationPolicy::Regular/);
  assert.match(processManager, /\.kill_on_drop\(true\)/);
});
