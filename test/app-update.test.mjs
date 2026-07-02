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
  assert.match(prompt, /downloadAppUpdate/);
  assert.match(prompt, /openAppUpdateInstaller/);
  assert.match(prompt, /installerPath/);
  assert.match(prompt, /Open/);
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
  assert.match(commands, /downloaded: installer_path\.exists\(\)/);
  assert.match(commands, /installer_path/);
  assert.match(github, /select_app_platform_asset/);
  assert.match(github, /FRP-Manager_/);
  assert.match(types, /downloadUrl: string;/);
  assert.match(types, /assetName: string;/);
  assert.match(types, /downloaded: boolean;/);
  assert.match(types, /installerPath: string;/);
  assert.match(api, /downloadAppUpdate/);
  assert.match(api, /openAppUpdateInstaller/);
  assert.match(api, /invokeOrFallback\("download_app_update"/);
  assert.match(api, /invokeOrFallback\("open_app_update_installer"/);
  assert.match(lib, /commands::download_app_update/);
  assert.match(lib, /commands::open_app_update_installer/);
});

test("app update prompt downloads first and opens existing installers manually", async () => {
  const prompt = await readFile(
    new URL("../src/components/AppUpdatePrompt.tsx", import.meta.url),
    "utf8",
  );

  assert.match(prompt, /setInstallerPath\(update\.downloaded \? update\.installerPath : null\)/);
  assert.match(prompt, /await downloadAppUpdate\(\)/);
  assert.match(prompt, /setInstallerPath\(result\.installerPath\)/);
  assert.match(prompt, /await openAppUpdateInstaller\(\)/);
  assert.match(prompt, /installerPath \? "Open" : "Download"/);
  assert.doesNotMatch(prompt, /onClose\(\);\s*\}\s*catch[\s\S]*downloadAppUpdate/);
});

test("app update modal action buttons do not overflow the dialog", async () => {
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  const modalBlock = css.match(
    /\.app-update-modal\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;
  const actionsBlock = css.match(
    /\.app-update-actions\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;
  const buttonBlock = css.match(
    /\.app-update-actions \.command-button\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;

  assert.ok(modalBlock, "expected app update modal CSS block");
  assert.ok(actionsBlock, "expected app update actions CSS block");
  assert.ok(buttonBlock, "expected app update action button CSS block");
  assert.match(modalBlock, /width:\s*min\(560px,\s*calc\(100vw - 48px\)\)/);
  assert.match(modalBlock, /overflow-x:\s*hidden/);
  assert.match(actionsBlock, /display:\s*grid/);
  assert.match(actionsBlock, /grid-template-columns:\s*minmax\(0,\s*0\.75fr\)\s+minmax\(0,\s*1\.35fr\)\s+minmax\(0,\s*1\.15fr\)/);
  assert.match(buttonBlock, /min-width:\s*0/);
  assert.match(buttonBlock, /width:\s*100%/);
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
