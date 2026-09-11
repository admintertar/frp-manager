import { Pencil, Plus, Trash2 } from "lucide-react";
import { useTranslation } from "../lib/i18n";
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
  const t = useTranslation();
  const isProxyBusy = busyProxyName !== null && busyProxyName !== undefined;

  return (
    <section className="table-panel">
      <div className="panel-header">
        <strong>{t("table.title")}</strong>
        <button className="command-button compact" onClick={onAdd}>
          <Plus size={15} /> {t("table.addProxy")}
        </button>
      </div>
      <div className="proxy-table-header">
        <span>{t("table.columnName")}</span>
        <span>{t("table.columnType")}</span>
        <span>{t("table.columnLocal")}</span>
        <span>{t("table.columnRemote")}</span>
        <span>{t("table.columnStatus")}</span>
        <span>{t("table.columnActions")}</span>
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
              aria-label={t("table.enable", { name: proxy.name })}
              onChange={(event) =>
                void onToggle(proxy.name, event.target.checked)
              }
            />
            <span className={proxy.enabled ? "proxy-online" : "proxy-offline"}>
              {proxy.enabled ? t("table.online") : t("table.off")}
            </span>
          </label>
          <div className="proxy-row-actions">
            <button
              className="icon-button table-icon-button"
              type="button"
              aria-label={t("table.editProxy", { name: proxy.name })}
              disabled={isProxyBusy}
              onClick={() => onEdit(proxy)}
            >
              <Pencil size={14} />
            </button>
            <button
              className="icon-button table-icon-button danger-icon"
              type="button"
              aria-label={t("table.deleteProxy", { name: proxy.name })}
              disabled={isProxyBusy}
              onClick={() => void onDelete(proxy.name)}
            >
              <Trash2 size={14} />
            </button>
          </div>
        </div>
      ))}
      {proxies.length === 0 ? (
        <div className="proxy-table-empty">{t("table.empty")}</div>
      ) : null}
    </section>
  );
}

function formatLocal(proxy: ProxyConfig): string {
  if (!proxy.localPort) return "-";
  return `${proxy.localIp ?? "127.0.0.1"}:${proxy.localPort}`;
}
