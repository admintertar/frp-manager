import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("FRP Manager app update checks use this app GitHub releases", async () => {
  const github = await readFile(
    new URL("../src-tauri/src/github_release.rs", import.meta.url),
    "utf8",
  );
  const commands = await readFile(
    new URL("../src-tauri/src/commands.rs", import.meta.url),
    "utf8",
  );
  const lib = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const api = await readFile(new URL("../src/lib/api.ts", import.meta.url), "utf8");

  assert.match(github, /APP_LATEST_RELEASE_URL/);
  assert.match(github, /github\.com\/admintertar\/frp-manager\/releases\/latest/);
  assert.doesNotMatch(github, /api\.github\.com\/repos\/admintertar\/frp-manager/);
  assert.match(github, /app_release_tag_from_latest_url/);
  assert.match(commands, /check_app_update/);
  assert.match(commands, /app_update_available/);
  assert.match(lib, /commands::check_app_update/);
  assert.match(api, /checkAppUpdate/);
  assert.match(api, /invokeOrFallback\("check_app_update"/);
});

test("frpc runtime update checks do not use GitHub releases API", async () => {
  const github = await readFile(
    new URL("../src-tauri/src/github_release.rs", import.meta.url),
    "utf8",
  );

  assert.match(github, /github\.com\/fatedier\/frp\/releases\/latest/);
  assert.doesNotMatch(github, /api\.github\.com\/repos\/fatedier\/frp\/releases\/latest/);
  assert.match(github, /frp_release_from_latest_url/);
  assert.match(github, /frp_sha256_checksums\.txt/);
});

test("app update prompt is a small modal with update and ignore actions", async () => {
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const prompt = await readFile(
    new URL("../src/components/AppUpdatePrompt.tsx", import.meta.url),
    "utf8",
  );
  const tray = await readFile(new URL("../src-tauri/src/tray.rs", import.meta.url), "utf8");

  assert.match(app, /AppUpdatePrompt/);
  assert.match(app, /APP_UPDATE_IGNORED_VERSION_KEY/);
  assert.match(app, /listenAppUpdateCheckRequested/);
  assert.match(prompt, /New version available/);
  assert.match(prompt, /Ignore this version/);
  assert.match(prompt, /installAppUpdate/);
  assert.doesNotMatch(prompt, /openUrl\(update\.releaseUrl\)/);
  assert.match(tray, /APP_UPDATE_CHECK_REQUESTED_EVENT/);
  assert.match(tray, /app-update-check-requested/);
});

test("app update check returns a direct installer download url", async () => {
  const commands = await readFile(
    new URL("../src-tauri/src/commands.rs", import.meta.url),
    "utf8",
  );
  const github = await readFile(
    new URL("../src-tauri/src/github_release.rs", import.meta.url),
    "utf8",
  );
  const types = await readFile(new URL("../src/types.ts", import.meta.url), "utf8");
  const api = await readFile(new URL("../src/lib/api.ts", import.meta.url), "utf8");
  const lib = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

  assert.match(commands, /download_url: release\.download_url/);
  assert.match(github, /select_app_platform_asset/);
  assert.match(github, /FRP-Manager_/);
  assert.match(types, /downloadUrl: string;/);
  assert.match(types, /assetName: string;/);
  assert.match(api, /installAppUpdate/);
  assert.match(api, /invokeOrFallback\("install_app_update"/);
  assert.match(lib, /commands::install_app_update/);
});

test("manual app update checks show feedback when already current", async () => {
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const prompt = await readFile(
    new URL("../src/components/AppUpdatePrompt.tsx", import.meta.url),
    "utf8",
  );

  assert.match(
    app,
    /if \(!update\.updateAvailable\) \{\s*if \(manual\) \{\s*setAppUpdate\(update\);\s*setAppUpdateOpen\(true\);/s,
  );
  assert.match(prompt, /FRP Manager is up to date/);
  assert.match(prompt, /update\.updateAvailable \?/);
});
