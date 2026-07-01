import { Pencil, Plus, Trash2 } from "lucide-react";
import { formatRemoteAddress } from "../lib/remoteDisplay";
import type { ProxyConfig } from "../types";

interface Props {
  proxies: ProxyConfig[];
  serverAddr: string;
  busyProxyName?: string | null;
  onAdd: () => void;
  onEdit: (proxy: ProxyConfig) => void;
  onDelete: (proxyName: string) => Promise<void>;
  onToggle: (proxyName: string, enabled: boolean) => Promise<void>;
}

export function ProxyTable({
  proxies,
  serverAddr,
  busyProxyName,
  onAdd,
  onEdit,
  onDelete,
  onToggle,
}: Props) {
  const isProxyBusy = busyProxyName !== null && busyProxyName !== undefined;

  return (
    <section className="table-panel">
      <div className="panel-header">
        <strong>Proxies</strong>
        <button className="command-button compact" onClick={onAdd}>
          <Plus size={15} /> Add Proxy
        </button>
      </div>
      <div className="proxy-table-header">
        <span>Name</span>
        <span>Type</span>
        <span>Local</span>
        <span>Remote</span>
        <span>Status</span>
        <span>Actions</span>
      </div>
      {proxies.map((proxy) => (
        <div className="proxy-table-row" key={proxy.name}>
          <strong>{proxy.name}</strong>
          <span>{proxy.proxyType}</span>
          <span>{formatLocal(proxy)}</span>
          <span>{formatRemoteAddress(proxy, serverAddr)}</span>
          <label className="proxy-status-toggle">
            <input
              type="checkbox"
              checked={proxy.enabled}
              disabled={isProxyBusy}
              aria-label={`Enable ${proxy.name}`}
              onChange={(event) =>
                void onToggle(proxy.name, event.target.checked)
              }
            />
            <span className={proxy.enabled ? "proxy-online" : "proxy-offline"}>
              {proxy.enabled ? "online" : "off"}
            </span>
          </label>
          <div className="proxy-row-actions">
            <button
              className="icon-button table-icon-button"
              type="button"
              aria-label={`Edit ${proxy.name}`}
              disabled={isProxyBusy}
              onClick={() => onEdit(proxy)}
            >
              <Pencil size={14} />
            </button>
            <button
              className="icon-button table-icon-button danger-icon"
              type="button"
              aria-label={`Delete ${proxy.name}`}
              disabled={isProxyBusy}
              onClick={() => void onDelete(proxy.name)}
            >
              <Trash2 size={14} />
            </button>
          </div>
        </div>
      ))}
      {proxies.length === 0 ? (
        <div className="proxy-table-empty">This Profile has no proxies.</div>
      ) : null}
    </section>
  );
}

function formatLocal(proxy: ProxyConfig): string {
  if (!proxy.localPort) return "-";
  return `${proxy.localIp ?? "127.0.0.1"}:${proxy.localPort}`;
}
