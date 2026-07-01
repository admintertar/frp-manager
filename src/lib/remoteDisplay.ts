import type { ProxyConfig } from "../types";

export function formatRemoteAddress(
  proxy: ProxyConfig,
  serverAddr: string,
): string {
  const host = normalizeHost(serverAddr);

  if (proxy.proxyType === "tcp" || proxy.proxyType === "udp") {
    if (!host || !proxy.remotePort) return "-";
    return `${proxy.proxyType}://${host}:${proxy.remotePort}`;
  }

  const addresses = proxy.customDomains
    .map((domain) => normalizeHost(domain))
    .filter(Boolean)
    .map((domain) => `${proxy.proxyType}://${domain}`);

  const subdomain = proxy.subdomain?.trim();
  if (subdomain && host) {
    addresses.push(`${proxy.proxyType}://${subdomain}.${host}`);
  }

  return addresses.length > 0 ? addresses.join(", ") : "-";
}

function normalizeHost(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) return "";

  const withoutScheme = trimmed.replace(/^[a-z][a-z\d+.-]*:\/\//i, "");
  return withoutScheme.split(/[/?#]/, 1)[0]?.replace(/\/+$/, "") ?? "";
}
