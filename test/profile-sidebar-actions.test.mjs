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

test("context menu labels never wrap", async () => {
  const source = await readFile(
    new URL("../src/components/ProfileSidebar.tsx", import.meta.url),
    "utf8",
  );
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");

  function cssBlock(selector) {
    const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return css.match(new RegExp(`${escaped}\\s*\\{(?<body>[\\s\\S]*?)\\}`))?.groups
      ?.body;
  }

  const menuBlock = cssBlock(".profile-context-menu");
  const itemBlock = cssBlock(".profile-context-menu button");

  assert.ok(menuBlock, "expected context menu CSS block");
  assert.ok(itemBlock, "expected context menu item CSS block");
  // A fixed width clipped the label; the longest one wrapped onto a second line
  // once it had to share the row with the check mark column.
  assert.match(menuBlock, /width:\s*max-content/);
  assert.match(menuBlock, /min-width:\s*152px/);
  // No plain fixed width, which is what clipped the label in the first place.
  assert.doesNotMatch(menuBlock, /(?<!min-)(?<!max-)\bwidth:\s*[\d.]+px/);
  assert.match(itemBlock, /white-space:\s*nowrap/);

  // The open position used to be clamped with a hardcoded menu size, which goes
  // stale as soon as the labels change. The menu measures itself instead.
  assert.match(source, /menuRef/);
  assert.match(source, /getBoundingClientRect\(\)/);
  assert.doesNotMatch(source, /window\.innerWidth - 168/);
});
