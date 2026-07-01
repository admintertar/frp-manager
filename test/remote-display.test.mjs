import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import ts from "typescript";

const source = await readFile(
  new URL("../src/lib/remoteDisplay.ts", import.meta.url),
  "utf8",
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ES2020,
    target: ts.ScriptTarget.ES2020,
  },
});
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled.outputText).toString("base64")}`;
const { formatRemoteAddress } = await import(moduleUrl);

test("formats http subdomains as full remote addresses", () => {
  assert.equal(
    formatRemoteAddress(
      {
        name: "zwd",
        proxyType: "http",
        enabled: true,
        localIp: "127.0.0.1",
        localPort: 8123,
        remotePort: null,
        subdomain: "zwd",
        customDomains: [],
      },
      "frp.ala4.com",
    ),
    "http://zwd.frp.ala4.com",
  );
});

test("formats https custom domains with scheme", () => {
  assert.equal(
    formatRemoteAddress(
      {
        name: "site",
        proxyType: "https",
        enabled: true,
        localIp: "127.0.0.1",
        localPort: 443,
        remotePort: null,
        subdomain: null,
        customDomains: ["zwd.frp.ala4.com", "api.frp.ala4.com"],
      },
      "frp.ala4.com",
    ),
    "https://zwd.frp.ala4.com, https://api.frp.ala4.com",
  );
});

test("formats tcp remote ports with server address", () => {
  assert.equal(
    formatRemoteAddress(
      {
        name: "ssh",
        proxyType: "tcp",
        enabled: true,
        localIp: "127.0.0.1",
        localPort: 22,
        remotePort: 6000,
        subdomain: null,
        customDomains: [],
      },
      "frp.ala4.com",
    ),
    "tcp://frp.ala4.com:6000",
  );
});
