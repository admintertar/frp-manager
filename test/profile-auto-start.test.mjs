import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("profile summaries expose the auto start flag end to end", async () => {
  const types = await readFile(
    new URL("../src/types.ts", import.meta.url),
    "utf8",
  );
  const summary = types.match(/export interface ProfileSummary \{[\s\S]*?\n\}/);
  assert.ok(summary, "ProfileSummary interface should exist");
  assert.match(summary[0], /autoStart: boolean;/);
});

test("the auto start toggle is wired through the sidebar", async () => {
  const sidebar = await readFile(
    new URL("../src/components/ProfileSidebar.tsx", import.meta.url),
    "utf8",
  );
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");

  assert.match(sidebar, /onToggleAutoStart/);
  assert.match(sidebar, /role="menuitemcheckbox"/);
  assert.match(sidebar, /aria-checked=\{menuProfile\.autoStart\}/);
  assert.match(
    sidebar,
    /onToggleAutoStart\(menuProfile\.id, !menuProfile\.autoStart\)/,
  );
  assert.match(sidebar, /profile-row-auto/);

  assert.match(app, /handleToggleAutoStart/);
  assert.match(app, /onToggleAutoStart=\{\(profileId, autoStart\)/);
});

test("the auto start command is exposed to the frontend", async () => {
  const api = await readFile(new URL("../src/lib/api.ts", import.meta.url), "utf8");

  assert.match(api, /export function setProfileAutoStart\(/);
  assert.match(api, /"set_profile_auto_start"/);
});

test("the backend registers and applies auto start", async () => {
  const lib = await readFile(
    new URL("../src-tauri/src/lib.rs", import.meta.url),
    "utf8",
  );
  const models = await readFile(
    new URL("../src-tauri/src/models.rs", import.meta.url),
    "utf8",
  );
  const commands = await readFile(
    new URL("../src-tauri/src/commands.rs", import.meta.url),
    "utf8",
  );

  assert.match(lib, /commands::set_profile_auto_start/);
  assert.match(lib, /spawn_auto_start/);
  assert.match(models, /pub auto_start: bool,/);
  assert.match(commands, /pub async fn set_profile_auto_start\(/);
  assert.match(commands, /pub async fn start_auto_start_profiles\(/);
});
