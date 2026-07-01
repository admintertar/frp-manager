import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("profile sidebar exposes edit and delete through a context menu", async () => {
  const source = await readFile(
    new URL("../src/components/ProfileSidebar.tsx", import.meta.url),
    "utf8",
  );

  assert.match(source, /onContextMenu/);
  assert.match(source, /role="menu"/);
  assert.match(source, /onEditProfile/);
  assert.match(source, /onDeleteProfile/);
  assert.doesNotMatch(source, /aria-label=\{`Edit profile/);
  assert.doesNotMatch(source, /aria-label=\{`Delete profile/);
});
