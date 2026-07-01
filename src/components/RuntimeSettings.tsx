import {
  AlertCircle,
  CheckCircle2,
  Download,
  FolderOpen,
  RefreshCw,
  X,
} from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useEffect, useRef, useState } from "react";
import { checkRuntimeUpdate, installRuntime } from "../lib/api";
import type { RuntimeStatus, RuntimeUpdateCheck } from "../types";

interface Props {
  open: boolean;
  status: RuntimeStatus;
  onClose: () => void;
  onStatusChange: (status: RuntimeStatus) => void;
}

export function RuntimeSettings({
  open,
  status,
  onClose,
  onStatusChange,
}: Props) {
  const [result, setResult] = useState<RuntimeUpdateCheck | null>(null);
  const [localStatus, setLocalStatus] = useState<RuntimeStatus>(status);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<"check" | "install" | "folder" | null>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const onCloseRef = useRef(onClose);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (!open) return;
    setLocalStatus(status);

    const previouslyFocused =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    closeButtonRef.current?.focus();

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onCloseRef.current();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      previouslyFocused?.focus();
    };
  }, [open]);

  if (!open) return null;

  async function check() {
    if (busy) return;
    setBusy("check");
    setError(null);
    try {
      setResult(await checkRuntimeUpdate());
    } catch (err) {
      setError(formatError(err));
    } finally {
      setBusy(null);
    }
  }

  async function install() {
    if (busy) return;
    setBusy("install");
    setError(null);
    try {
      const next = await installRuntime();
      setLocalStatus(next);
      onStatusChange(next);
      setResult(null);
    } catch (err) {
      setError(formatError(err));
    } finally {
      setBusy(null);
    }
  }

  async function openFolder() {
    if (!localStatus.runtimePath || busy) return;
    setBusy("folder");
    setError(null);
    try {
      await revealItemInDir(localStatus.runtimePath);
    } catch (err) {
      setError(formatError(err));
    } finally {
      setBusy(null);
    }
  }

  const installed = localStatus.installed;
  const statusLabel = result
    ? result.updateAvailable
      ? "Update available"
      : "Runtime is current"
    : installed
      ? "Runtime installed"
      : "frpc runtime not installed";
  const statusClass = !installed
    ? "runtime-status runtime-status-missing"
    : result?.updateAvailable
      ? "runtime-status runtime-status-update"
      : "runtime-status";
  const actionLabel = installed ? "Update" : "Download";

  return (
    <div className="modal-backdrop">
      <section
        className="modal runtime-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="runtime-settings-title"
      >
        <header className="modal-title-row">
          <div>
            <h2 id="runtime-settings-title">frpc Runtime</h2>
            <p className="muted">fatedier/frp GitHub Releases</p>
          </div>
          <button
            ref={closeButtonRef}
            type="button"
            className="icon-button"
            aria-label="Close runtime settings"
            onClick={onClose}
          >
            <X size={18} />
          </button>
        </header>

        {error ? <div className="error-banner">{error}</div> : null}

        <div className={statusClass}>
          {!installed || result?.updateAvailable ? (
            <AlertCircle size={18} />
          ) : (
            <CheckCircle2 size={18} />
          )}
          <strong>{statusLabel}</strong>
        </div>

        <div className="runtime-result-grid">
          <div>
            <span>Current</span>
            <strong>
              {result?.currentVersion ??
                localStatus.currentVersion ??
                "not installed"}
            </strong>
          </div>
          <div>
            <span>Latest</span>
            <strong>{result?.latestVersion ?? "-"}</strong>
          </div>
          <div>
            <span>Platform</span>
            <strong>
              {localStatus.platform.os} {localStatus.platform.arch}
            </strong>
          </div>
          <div className="runtime-asset">
            <span>Asset</span>
            <strong>{result?.assetName ?? "-"}</strong>
          </div>
          <div className="runtime-asset">
            <span>Path</span>
            <strong>{localStatus.runtimePath ?? "-"}</strong>
          </div>
        </div>

        <div className="modal-actions">
          <button type="button" className="command-button" onClick={onClose}>
            Close
          </button>
          <button
            type="button"
            className="command-button"
            disabled={!localStatus.runtimePath || busy !== null}
            onClick={() => void openFolder()}
          >
            <FolderOpen size={16} />
            Open Folder
          </button>
          <button
            type="button"
            className="command-button"
            disabled={busy !== null}
            onClick={() => void check()}
          >
            <RefreshCw size={16} />
            {busy === "check" ? "Checking" : "Check Updates"}
          </button>
          <button
            type="button"
            className="command-button primary"
            disabled={busy !== null}
            onClick={() => void install()}
          >
            <Download size={16} />
            {busy === "install" ? "Downloading" : actionLabel}
          </button>
        </div>
      </section>
    </div>
  );
}

function formatError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "object" && err !== null && "message" in err) {
    const message = (err as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return String(err);
}
