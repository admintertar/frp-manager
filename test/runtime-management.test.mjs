import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("runtime management commands are exposed to the frontend", async () => {
  const lib = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const api = await readFile(new URL("../src/lib/api.ts", import.meta.url), "utf8");

  assert.match(lib, /commands::get_runtime_status/);
  assert.match(lib, /commands::install_runtime/);
  assert.match(api, /invokeOrFallback\("get_runtime_status"/);
  assert.match(api, /invokeOrFallback\("install_runtime"/);
});

test("runtime settings exposes install and update management states", async () => {
  const settings = await readFile(
    new URL("../src/components/RuntimeSettings.tsx", import.meta.url),
    "utf8",
  );
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const workbench = await readFile(
    new URL("../src/components/ProfileWorkbench.tsx", import.meta.url),
    "utf8",
  );

  assert.match(settings, /installRuntime/);
  assert.match(settings, /t\("runtimeSettings\.notInstalled/);
  assert.match(settings, /Download/);
  assert.match(settings, /t\("runtimeSettings\.openFolder"\)/);
  assert.match(app, /getRuntimeStatus/);
  assert.match(workbench, /runtimeInstalled/);
});

test("runtime settings refreshes local status when the modal opens", async () => {
  const settings = await readFile(
    new URL("../src/components/RuntimeSettings.tsx", import.meta.url),
    "utf8",
  );

  assert.match(settings, /getRuntimeStatus/);
  assert.match(settings, /refreshStatus/);
  assert.match(settings, /onStatusChange\(next\)/);
});

test("runtime settings fills latest release and asset details when opened", async () => {
  const settings = await readFile(
    new URL("../src/components/RuntimeSettings.tsx", import.meta.url),
    "utf8",
  );

  assert.match(settings, /refreshRuntimeDetails/);
  assert.match(settings, /await checkRuntimeUpdate\(\)/);
  assert.match(settings, /setResult\(update\)/);
  assert.match(settings, /result\?\.latestVersion/);
  assert.match(settings, /result\?\.assetName/);
});

test("runtime settings shows errors in the status card instead of a top banner", async () => {
  const settings = await readFile(
    new URL("../src/components/RuntimeSettings.tsx", import.meta.url),
    "utf8",
  );

  assert.doesNotMatch(settings, /<div className="error-banner">\{error\}<\/div>/);
  assert.match(settings, /runtime-status-error/);
  assert.match(settings, /statusMessage/);
});

test("runtime settings actions stay in one evenly distributed row", async () => {
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  const modalBlock = css.match(
    /\.runtime-modal\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;
  const actionsBlock = css.match(
    /\.runtime-modal \.modal-actions\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;
  const buttonBlock = css.match(
    /\.runtime-modal \.modal-actions \.command-button\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;

  assert.ok(modalBlock, "expected runtime modal CSS block");
  assert.ok(actionsBlock, "expected runtime modal action CSS block");
  assert.ok(buttonBlock, "expected runtime modal action button CSS block");
  assert.match(modalBlock, /width:\s*min\(720px,\s*calc\(100vw - 48px\)\)/);
  assert.match(actionsBlock, /display:\s*grid/);
  assert.match(actionsBlock, /grid-template-columns:\s*repeat\(4,\s*minmax\(0,\s*1fr\)\)/);
  assert.match(buttonBlock, /min-width:\s*0/);
  assert.match(buttonBlock, /white-space:\s*nowrap/);
});

test("disabled primary command buttons keep their primary hover treatment", async () => {
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  const disabledPrimaryHover = css.match(
    /\.command-button\.primary:disabled:hover\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;

  assert.ok(disabledPrimaryHover, "expected disabled primary hover CSS block");
  assert.match(disabledPrimaryHover, /background:\s*var\(--blue\)/);
  assert.match(disabledPrimaryHover, /border-color:\s*var\(--blue\)/);
  assert.match(disabledPrimaryHover, /color:\s*#ffffff/);
});

test("runtime settings close action uses an icon", async () => {
  const settings = await readFile(
    new URL("../src/components/RuntimeSettings.tsx", import.meta.url),
    "utf8",
  );

  assert.match(settings, /<X size=\{16\} \/>[\s\S]*t\("common\.close"\)/);
});

test("frpc is not bundled as a Tauri sidecar", async () => {
  const config = await readFile(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
    "utf8",
  );

  assert.doesNotMatch(config, /externalBin/);
  assert.doesNotMatch(config, /binaries\/frpc/);
});
