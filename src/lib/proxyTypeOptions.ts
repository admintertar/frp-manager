import type { ProxyType } from "../types";

export function proxyTypeOptions(
  mode: "add" | "edit",
  currentType: ProxyType,
): ProxyType[] {
  const options: ProxyType[] = ["http", "tcp", "udp"];
  if (mode === "edit" && currentType === "https") {
    options.splice(1, 0, "https");
  }
  return options;
}
