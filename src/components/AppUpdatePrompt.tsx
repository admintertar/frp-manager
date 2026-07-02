import { Download, ExternalLink, X } from "lucide-react";
import { useEffect, useState } from "react";
import { downloadAppUpdate, openAppUpdateInstaller } from "../lib/api";
import type { AppUpdateCheck } from "../types";

interface Props {
  update: AppUpdateCheck | null;
  open: boolean;
  onClose: () => void;
  onIgnore: (version: string) => void;
}

export function AppUpdatePrompt({ update, open, onClose, onIgnore }: Props) {
  const [busy, setBusy] = useState<"download" | "open" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [installerPath, setInstallerPath] = useState<string | null>(null);

  useEffect(() => {
    setError(null);
    setBusy(null);
    if (!open || !update) {
      setInstallerPath(null);
      return;
    }
    setInstallerPath(update.downloaded ? update.installerPath : null);
  }, [open, update?.assetName, update?.downloaded, update?.installerPath]);

  if (!open || !update) return null;
  const title = update.updateAvailable
    ? "New version available"
    : "FRP Manager is up to date";
  const latestLabel = update.updateAvailable
    ? `FRP Manager ${update.latestVersion}`
    : "No newer release was found";
  const idlePrimaryLabel = installerPath ? "Open" : "Download";
  const primaryLabel = busy === "download" ? "Downloading..." : idlePrimaryLabel;

  async function handlePrimaryAction() {
    if (!update) return;
    const nextAction = installerPath ? "open" : "download";
    setBusy(nextAction);
    setError(null);
    try {
      if (installerPath) {
        await openAppUpdateInstaller();
      } else {
        const result = await downloadAppUpdate();
        setInstallerPath(result.installerPath);
      }
    } catch (err) {
      setError(formatError(err));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="modal-backdrop">
      <section
        className="modal app-update-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="app-update-title"
      >
        <header className="modal-title-row">
          <div>
            <h2 id="app-update-title">{title}</h2>
            <p className="muted">{latestLabel}</p>
          </div>
          <button
            type="button"
            className="icon-button"
            aria-label="Close update prompt"
            onClick={onClose}
          >
            <X size={18} />
          </button>
        </header>

        <div className="app-update-copy">
          <span>Current {update.currentVersion}</span>
          <strong>Latest {update.latestVersion}</strong>
        </div>
        {error ? <div className="error-banner">{error}</div> : null}

        <div
          className={`modal-actions app-update-actions ${
            update.updateAvailable ? "" : "app-update-actions-single"
          }`}
        >
          <button
            type="button"
            className="command-button"
            onClick={onClose}
            disabled={busy !== null}
          >
            {update.updateAvailable ? "Later" : "Close"}
          </button>
          {update.updateAvailable ? (
            <>
              <button
                type="button"
                className="command-button"
                onClick={() => onIgnore(update.latestVersion)}
                disabled={busy !== null}
              >
                Ignore this version
              </button>
              <button
                type="button"
                className="command-button primary"
                onClick={() => void handlePrimaryAction()}
                disabled={busy !== null}
              >
                {installerPath ? <ExternalLink size={16} /> : <Download size={16} />}
                {primaryLabel}
              </button>
            </>
          ) : null}
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
