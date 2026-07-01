import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("profile workbench distinguishes loading detail from no selection", async () => {
  const source = await readFile(
    new URL("../src/components/ProfileWorkbench.tsx", import.meta.url),
    "utf8",
  );

  assert.match(source, /selectedProfile\?: ProfileSummary/);
  assert.match(source, /Loading profile/);
  assert.match(source, /profile-loading-panel/);
  assert.match(source, /profile \?\?/);
});

test("profile selection does not clear detail before loading the next profile", async () => {
  const source = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const selectionEffect = source.match(
    /useEffect\(\(\) => \{[\s\S]*?\}, \[selected\?\.id\]\);/,
  )?.[0] ?? "";

  assert.doesNotMatch(
    selectionEffect,
    /selectedIdRef\.current = requestedId;\s*setProfileDetail\(undefined\)/,
  );
  assert.doesNotMatch(
    selectionEffect,
    /selectedIdRef\.current = requestedId;[\s\S]*?void loadProfileDetail\(requestedId\)/,
  );
  assert.match(source, /async function fetchProfileDetail/);
  assert.match(source, /const detail = await fetchProfileDetail\(profileId\)/);
  assert.match(source, /pendingSelectionRef/);
  assert.doesNotMatch(
    source,
    /async function handleSelectProfile[\s\S]*?selectedIdRef\.current = profileId;[\s\S]*?const detail = await fetchProfileDetail\(profileId\)/,
  );
});
