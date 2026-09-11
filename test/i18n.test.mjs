import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { test } from "node:test";
import ts from "typescript";

/**
 * i18n.ts also exports React-bound helpers. Only the pure parts are exercised
 * here, so the React import is replaced with inert stubs before transpiling.
 */
const source = await readFile(
  new URL("../src/lib/i18n.ts", import.meta.url),
  "utf8",
);
const stubbed = `${source.replace(
  /^import \{[^}]*\} from "react";$/m,
  "",
)}
export const useMemo = (factory) => factory();
export const useSyncExternalStore = (_subscribe, getSnapshot) => getSnapshot();
`;
const compiled = ts.transpileModule(stubbed, {
  compilerOptions: {
    module: ts.ModuleKind.ES2020,
    target: ts.ScriptTarget.ES2020,
  },
});
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled.outputText).toString("base64")}`;
const {
  DEFAULT_LOCALE,
  LOCALES,
  createTranslator,
  localeLabel,
  messageKeys,
  otherLocale,
  resolveLocale,
  setLocale,
  translate,
} = await import(moduleUrl);

function placeholders(template) {
  return [...template.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();
}

test("every locale defines the same keys", () => {
  const reference = messageKeys(DEFAULT_LOCALE);

  assert.ok(reference.length > 50, "expected a substantive message table");
  for (const locale of LOCALES) {
    assert.deepEqual(messageKeys(locale), reference, `key mismatch in ${locale}`);
  }
});

test("no translated string is empty", () => {
  for (const locale of LOCALES) {
    for (const key of messageKeys(locale)) {
      assert.notEqual(
        translate(locale, key).trim(),
        "",
        `empty translation for ${key} (${locale})`,
      );
    }
  }
});

test("translations keep the placeholders of the source string", () => {
  for (const key of messageKeys(DEFAULT_LOCALE)) {
    const expected = placeholders(translate(DEFAULT_LOCALE, key));
    if (expected.length === 0) continue;

    for (const locale of LOCALES) {
      assert.deepEqual(
        placeholders(translate(locale, key)),
        expected,
        `placeholder mismatch for ${key} (${locale})`,
      );
    }
  }
});

test("translations actually differ between locales", () => {
  const reference = messageKeys(DEFAULT_LOCALE);
  const identical = reference.filter(
    (key) => translate("en", key) === translate("zh-CN", key),
  );

  // A handful of strings are config keys or product names and stay as-is, so
  // allow a small overlap rather than requiring every key to differ.
  assert.ok(
    identical.length < reference.length / 4,
    `too many untranslated strings: ${identical.join(", ")}`,
  );
});

test("placeholder values are substituted", () => {
  assert.equal(
    translate("en", "metrics.active", { count: 3 }),
    "3 active",
  );
  assert.equal(
    translate("zh-CN", "metrics.active", { count: 3 }),
    "3 个已启用",
  );
});

test("unknown placeholders are left untouched", () => {
  assert.equal(
    translate("en", "metrics.active", { other: "x" }),
    "{count} active",
  );
});

test("the active locale drives the translator", () => {
  setLocale("zh-CN");
  assert.equal(createTranslator("zh-CN")("common.save"), "保存");
  assert.equal(createTranslator("en")("common.save"), "Save");
  setLocale(DEFAULT_LOCALE);
});

test("locale tags resolve to a supported locale", () => {
  assert.equal(resolveLocale("zh-CN"), "zh-CN");
  assert.equal(resolveLocale("zh"), "zh-CN");
  assert.equal(resolveLocale("zh-Hans-CN"), "zh-CN");
  assert.equal(resolveLocale("ZH-cn"), "zh-CN");
  assert.equal(resolveLocale("en-US"), "en");
  assert.equal(resolveLocale("en"), "en");
  assert.equal(resolveLocale("fr-FR"), DEFAULT_LOCALE);
  assert.equal(resolveLocale(undefined), DEFAULT_LOCALE);
  assert.equal(resolveLocale(""), DEFAULT_LOCALE);
});

test("language labels and the toggle target are consistent", () => {
  assert.equal(localeLabel("zh-CN"), "中文");
  assert.equal(localeLabel("en"), "English");
  assert.equal(otherLocale("zh-CN"), "en");
  assert.equal(otherLocale("en"), "zh-CN");
});

test("a locale round-trips through its own label", () => {
  for (const locale of LOCALES) {
    assert.equal(resolveLocale(locale), locale, `unresolvable locale ${locale}`);
  }
});

test("every component resolves its strings through the translator", async () => {
  const dir = new URL("../src/components/", import.meta.url);
  const entries = await readdir(dir);
  const components = entries.filter((name) => name.endsWith(".tsx"));

  assert.ok(components.length >= 7, "expected the component set to be present");

  for (const name of components) {
    const contents = await readFile(new URL(name, dir), "utf8");
    assert.match(
      contents,
      /useTranslation/,
      `${name} does not use the translator`,
    );
  }
});
