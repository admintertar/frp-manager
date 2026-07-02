import { Download, X } from "lucide-react";
import { useState } from "react";
import { installAppUpdate } from "../lib/api";
import type { AppUpdateCheck } from "../types";

interface Props {
  update: AppUpdateCheck | null;
  open: boolean;
  onClose: () => void;
  onIgnore: (version: string) => void;
}

export function AppUpdatePrompt({ update, open, onClose, onIgnore }: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  if (!open || !update) return null;
  const title = update.updateAvailable
    ? "New version available"
    : "FRP Manager is up to date";
  const latestLabel = update.updateAvailable
    ? `FRP Manager ${update.latestVersion}`
    : "No newer release was found";

  async function updateNow() {
    if (!update) return;
    setBusy(true);
    setError(null);
    try {
      await installAppUpdate();
      onClose();
    } catch (err) {
      setError(formatError(err));
    } finally {
      setBusy(false);
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

        <div className="modal-actions app-update-actions">
          <button
            type="button"
            className="command-button"
            onClick={onClose}
            disabled={busy}
          >
            {update.updateAvailable ? "Later" : "Close"}
          </button>
          {update.updateAvailable ? (
            <>
              <button
                type="button"
                className="command-button"
                onClick={() => onIgnore(update.latestVersion)}
                disabled={busy}
              >
                Ignore this version
              </button>
              <button
                type="button"
                className="command-button primary"
                onClick={() => void updateNow()}
                disabled={busy}
              >
                <Download size={16} />
                {busy ? "Downloading..." : "Update"}
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
