import { useEffect, useMemo, useRef } from "react";
import { Edit3, Play, RefreshCw, RotateCw, Square } from "lucide-react";
import { parseAnsiLogLine, type AnsiLogSegment } from "../lib/ansiLog";
import { useTranslation } from "../lib/i18n";
import type { Profile, ProfileSummary, ProxyConfig, RuntimeState } from "../types";
import { ProxyTable } from "./ProxyTable";

interface Props {
  profile?: Profile;
  selectedProfile?: ProfileSummary;
  runtimeState: RuntimeState;
  runtimePid?: number | null;
  runtimeStartedAt?: string | null;
  runtimeInstalled: boolean;
  logs: string;
  error: string | null;
  busyAction?: string | null;
  busyProxyName?: string | null;
  onRefresh: () => Promise<void>;
  onReload: (profileId: string) => Promise<void>;
  onEdit: () => void;
  onAddProxy: () => void;
  onEditProxy: (proxy: ProxyConfig) => void;
  onDeleteProxy: (profileId: string, proxyName: string) => Promise<void>;
  onStart: (profileId: string) => Promise<void>;
  onStop: (profileId: string) => Promise<void>;
  onToggleProxy: (
    profileId: string,
    proxyName: string,
    enabled: boolean,
  ) => Promise<void>;
}

export function ProfileWorkbench({
  profile,
  selectedProfile,
  runtimeState,
  runtimePid,
  runtimeStartedAt,
  runtimeInstalled,
  logs,
  error,
  busyAction,
  busyProxyName,
  onRefresh,
  onReload,
  onEdit,
  onAddProxy,
  onEditProxy,
  onDeleteProxy,
  onStart,
  onStop,
  onToggleProxy,
}: Props) {
  const t = useTranslation();
  const activeProfile = profile ?? selectedProfile;
  const isProfileLoading = Boolean(selectedProfile && !profile);
  const authLabel = profile?.authMethod
    ? t("workbench.authMethod", { method: profile.authMethod })
    : t("workbench.authNotSet");
  const proxyActiveCount = profile?.proxies.filter((proxy) => proxy.enabled).length ?? 0;
  const actionBlockingControls =
    busyAction !== null &&
    busyAction !== undefined &&
    !isSettledRuntimeAction(
      busyAction,
      activeProfile?.id,
      runtimeState,
    );
  const busy = Boolean(actionBlockingControls);
  const logPanelRef = useRef<HTMLElement | null>(null);
  const logLines = useMemo(
    () => formatLogs(logs, t("log.waiting")),
    [logs, t],
  );

  useEffect(() => {
    const panel = logPanelRef.current;
    if (!panel) return;
    panel.scrollTop = panel.scrollHeight;
  }, [logLines]);

  return (
    <section className="workbench">
      <header className="workbench-header">
        <div>
          <h1>{activeProfile?.displayName ?? "FRP Manager"}</h1>
          <p>
            {profile
              ? t("workbench.detail", {
                  addr: profile.serverAddr,
                  port: profile.serverPort,
                  auth: authLabel,
                })
              : selectedProfile
                ? t("workbench.detail", {
                    addr: selectedProfile.serverAddr,
                    port: selectedProfile.serverPort,
                    auth: t("workbench.loadingProfile"),
                  })
                : t("workbench.importHint")}
          </p>
        </div>
        <div className="toolbar">
          {activeProfile && runtimeState === "running" ? (
            <button
              className="command-button danger"
              disabled={busy}
              onClick={() => void onStop(activeProfile.id)}
            >
              <Square size={13} fill="currentColor" strokeWidth={2.2} />{" "}
              {t("workbench.stop")}
            </button>
          ) : activeProfile ? (
            <button
              className="command-button primary"
              disabled={busy || !runtimeInstalled}
              title={
                runtimeInstalled
                  ? undefined
                  : t("workbench.installRuntimeFirst")
              }
              onClick={() => void onStart(activeProfile.id)}
            >
              <Play size={16} /> {t("workbench.start")}
            </button>
          ) : null}
          {activeProfile ? (
            <>
              <button
                className="command-button"
                disabled={busy || runtimeState !== "running" || isProfileLoading}
                onClick={() => void onReload(activeProfile.id)}
              >
                <RotateCw size={16} /> {t("workbench.reload")}
              </button>
              <button
                className="command-button"
                disabled={busy}
                onClick={onEdit}
              >
                <Edit3 size={16} /> {t("common.edit")}
              </button>
            </>
          ) : (
            <button
              className="icon-button"
              aria-label={t("workbench.refresh")}
              disabled={busy}
              onClick={() => void onRefresh()}
            >
              <RefreshCw size={18} />
            </button>
          )}
        </div>
      </header>

      {error ? <div className="error-banner">{error}</div> : null}

      <div className="metrics">
        <div>
          <span>{t("metrics.status")}</span>
          <strong className={`metric-state metric-${runtimeState}`}>
            {t(`state.${runtimeState}`)}
          </strong>
        </div>
        <div>
          <span>{t("metrics.proxies")}</span>
          <strong>
            {profile
              ? t("metrics.active", { count: proxyActiveCount })
              : selectedProfile
                ? t("metrics.configured", { count: selectedProfile.proxyCount })
                : t("metrics.active", { count: 0 })}
          </strong>
        </div>
        <div>
          <span>{t("metrics.pid")}</span>
          <strong>{runtimePid ?? "-"}</strong>
        </div>
        <div>
          <span>{t("metrics.uptime")}</span>
          <strong>{formatUptime(runtimeStartedAt, runtimeState)}</strong>
        </div>
      </div>

      {profile ? (
        <ProxyTable
          proxies={profile.proxies}
          serverAddr={profile.serverAddr}
          busyProxyName={busyProxyName}
          onAdd={onAddProxy}
          onEdit={onEditProxy}
          onDelete={(proxyName) => onDeleteProxy(profile.id, proxyName)}
          onToggle={(proxyName, enabled) =>
            onToggleProxy(profile.id, proxyName, enabled)
          }
        />
      ) : selectedProfile ? (
        <section className="empty-panel profile-loading-panel" aria-busy="true">
          <h2>{t("empty.loadingTitle")}</h2>
          <p>{t("empty.loadingBody", { name: selectedProfile.displayName })}</p>
        </section>
      ) : (
        <section className="empty-panel">
          <h2>{t("empty.noProfileTitle")}</h2>
          <p>{t("empty.noProfileBody")}</p>
        </section>
      )}

      <section className="log-panel" ref={logPanelRef}>
        {logLines.map((line, index) => (
          <div key={`${line}-${index}`}>
            {parseAnsiLogLine(line).map((segment, segmentIndex) => (
              <span
                className={logTokenClassName(segment)}
                key={`${segment.text}-${segmentIndex}`}
              >
                {segment.text}
              </span>
            ))}
          </div>
        ))}
      </section>
    </section>
  );
}

function isSettledRuntimeAction(
  action: string,
  profileId: string | undefined,
  runtimeState: RuntimeState,
): boolean {
  if (!profileId) return false;
  if (action === `start:${profileId}`) return runtimeState === "running";
  if (action === `stop:${profileId}`) return runtimeState !== "running";
  return false;
}

function formatUptime(
  startedAt: string | null | undefined,
  runtimeState: RuntimeState,
): string {
  if (!startedAt || runtimeState !== "running") return "-";
  const elapsedSeconds = Math.max(
    0,
    Math.floor((Date.now() - new Date(startedAt).getTime()) / 1000),
  );
  const hours = Math.floor(elapsedSeconds / 3600);
  const minutes = Math.floor((elapsedSeconds % 3600) / 60);
  const seconds = elapsedSeconds % 60;
  return [hours, minutes, seconds]
    .map((value) => String(value).padStart(2, "0"))
    .join(":");
}

function formatLogs(logs: string, placeholder: string): string[] {
  const lines = logs
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter(Boolean)
    .slice(-80);
  return lines.length > 0 ? lines : [placeholder];
}

function logTokenClassName(segment: AnsiLogSegment): string {
  const classNames = ["log-token"];
  if (segment.color) classNames.push(`log-token-${segment.color}`);
  if (segment.bold) classNames.push("log-token-bold");
  return classNames.join(" ");
}
