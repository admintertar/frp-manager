import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import ts from "typescript";

const source = await readFile(
  new URL("../src/lib/proxyTypeOptions.ts", import.meta.url),
  "utf8",
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ES2020,
    target: ts.ScriptTarget.ES2020,
  },
});
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled.outputText).toString("base64")}`;
const { proxyTypeOptions } = await import(moduleUrl);

test("add proxy type options do not include https", () => {
  assert.deepEqual(proxyTypeOptions("add", "http"), ["http", "tcp", "udp"]);
});

test("edit options retain https when editing an existing https proxy", () => {
  assert.deepEqual(proxyTypeOptions("edit", "https"), [
    "http",
    "https",
    "tcp",
    "udp",
  ]);
});

test("proxy editor uses a custom type control instead of native select", async () => {
  const source = await readFile(
    new URL("../src/components/ProxyEditor.tsx", import.meta.url),
    "utf8",
  );

  assert.doesNotMatch(source, /<select[\s>]/);
  assert.match(source, /className="proxy-type-options"/);
});

test("selected proxy type has a colored selected state", async () => {
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  const selectedBlock = css.match(
    /\.proxy-type-option\.selected\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;

  assert.ok(selectedBlock, "expected selected proxy type CSS block");
  assert.match(selectedBlock, /background:\s*var\(--blue\)/);
  assert.match(selectedBlock, /color:\s*#ffffff/);
});
