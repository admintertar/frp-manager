import { useEffect, useMemo, useRef } from "react";
import { Edit3, Play, RefreshCw, RotateCw, Square } from "lucide-react";
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
  const activeProfile = profile ?? selectedProfile;
  const isProfileLoading = Boolean(selectedProfile && !profile);
  const authLabel = profile?.authMethod ? `${profile.authMethod} auth` : "auth not set";
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
  const logLines = useMemo(() => formatLogs(logs), [logs]);

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
              ? `serverAddr: ${profile.serverAddr} · serverPort: ${profile.serverPort} · ${authLabel}`
              : selectedProfile
                ? `serverAddr: ${selectedProfile.serverAddr} · serverPort: ${selectedProfile.serverPort} · loading profile`
              : "Import a frpc profile to begin."}
          </p>
        </div>
        <div className="toolbar">
          {activeProfile && runtimeState === "running" ? (
            <button
              className="command-button danger"
              disabled={busy}
              onClick={() => void onStop(activeProfile.id)}
            >
              <Square size={13} fill="currentColor" strokeWidth={2.2} /> Stop
            </button>
          ) : activeProfile ? (
            <button
              className="command-button primary"
              disabled={busy || !runtimeInstalled}
              title={
                runtimeInstalled
                  ? undefined
                  : "Install frpc runtime before starting profiles"
              }
              onClick={() => void onStart(activeProfile.id)}
            >
              <Play size={16} /> Start
            </button>
          ) : null}
          {activeProfile ? (
            <>
              <button
                className="command-button"
                disabled={busy || runtimeState !== "running" || isProfileLoading}
                onClick={() => void onReload(activeProfile.id)}
              >
                <RotateCw size={16} /> Reload
              </button>
              <button
                className="command-button"
                disabled={busy}
                onClick={onEdit}
              >
                <Edit3 size={16} /> Edit
              </button>
            </>
          ) : (
            <button
              className="icon-button"
              aria-label="Refresh profiles"
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
          <span>Status</span>
          <strong className={`metric-state metric-${runtimeState}`}>
            {runtimeState}
          </strong>
        </div>
        <div>
          <span>Proxies</span>
          <strong>
            {profile
              ? `${proxyActiveCount} active`
              : selectedProfile
                ? `${selectedProfile.proxyCount} configured`
                : "0 active"}
          </strong>
        </div>
        <div>
          <span>PID</span>
          <strong>{runtimePid ?? "-"}</strong>
        </div>
        <div>
          <span>Uptime</span>
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
          <h2>Loading profile</h2>
          <p>Preparing {selectedProfile.displayName}.</p>
        </section>
      ) : (
        <section className="empty-panel">
          <h2>No Profile selected</h2>
          <p>Create a profile to start managing frpc.</p>
        </section>
      )}

      <section className="log-panel" ref={logPanelRef}>
        {logLines.map((line, index) => (
          <div key={`${line}-${index}`}>{line}</div>
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

function formatLogs(logs: string): string[] {
  const lines = logs
    .split(/\r?\n/)
    .map((line) => stripAnsiCodes(line).trimEnd())
    .filter(Boolean)
    .slice(-80);
  return lines.length > 0 ? lines : ["[I] waiting for frpc log output"];
}

function stripAnsiCodes(input: string): string {
  return input.replace(/\x1b(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])/g, "");
}
