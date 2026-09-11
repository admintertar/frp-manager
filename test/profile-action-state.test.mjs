import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("profile toolbar renders start and stop from runtime state only", async () => {
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const workbench = await readFile(
    new URL("../src/components/ProfileWorkbench.tsx", import.meta.url),
    "utf8",
  );

  assert.match(app, /busyAction=\{busyAction\}/);
  assert.match(workbench, /busyAction\?: string \| null/);
  assert.match(
    workbench,
    /const actionBlockingControls =\s+busyAction !== null &&\s+busyAction !== undefined &&\s+!isSettledRuntimeAction\(/,
  );
  assert.match(workbench, /const busy = Boolean\(actionBlockingControls\);/);
  assert.match(
    workbench,
    /function isSettledRuntimeAction\([\s\S]*action === `start:\$\{profileId\}`[\s\S]*runtimeState === "running"/,
  );
  assert.match(
    workbench,
    /function isSettledRuntimeAction\([\s\S]*action === `stop:\$\{profileId\}`[\s\S]*runtimeState !== "running"/,
  );
  assert.match(workbench, /runtimeState === "running"/);
  assert.match(app, /current === `start:\$\{id\}` && runtimeState === "running"/);
  assert.match(
    workbench,
    /className="command-button danger"[\s\S]*disabled=\{busy\}[\s\S]*onClick=\{\(\) => void onStop\(activeProfile\.id\)\}[\s\S]*t\("workbench\.stop"\)/,
  );
  assert.match(
    workbench,
    /className="command-button primary"[\s\S]*disabled=\{busy \|\| !runtimeInstalled\}[\s\S]*onClick=\{\(\) => void onStart\(activeProfile\.id\)\}[\s\S]*t\("workbench\.start"\)/,
  );
  assert.match(workbench, /<RotateCw size=\{16\} \/> \{t\("workbench\.reload"\)\}/);
  assert.doesNotMatch(workbench, /isStartActionComplete/);
  assert.doesNotMatch(workbench, /isStopActionComplete/);
  assert.doesNotMatch(workbench, /effectiveBusyAction/);
  assert.doesNotMatch(workbench, /isStarting/);
  assert.doesNotMatch(workbench, /isStopping/);
  assert.doesNotMatch(workbench, /isReloading/);
  assert.doesNotMatch(workbench, />\s*Starting\s*</);
  assert.doesNotMatch(workbench, />\s*Stopping\s*</);
  assert.doesNotMatch(workbench, />\s*Reloading\s*</);
  assert.match(app, /await refreshProfiles\(profileId,\s*true\)/);
  assert.doesNotMatch(
    functionSource(app, "handleStart", "handleStop"),
    /await refreshProfiles\(\);/,
  );
  assert.doesNotMatch(
    functionSource(app, "handleStop", "handleReload"),
    /await refreshProfiles\(\);/,
  );
});

function functionSource(source, startName, nextName) {
  const start = source.indexOf(`async function ${startName}`);
  const end = source.indexOf(`async function ${nextName}`, start + 1);
  assert.notEqual(start, -1, `${startName} should exist`);
  assert.notEqual(end, -1, `${nextName} should exist`);
  return source.slice(start, end);
}
