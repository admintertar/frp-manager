import { Download, X } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { AppUpdateCheck } from "../types";

interface Props {
  update: AppUpdateCheck | null;
  open: boolean;
  onClose: () => void;
  onIgnore: (version: string) => void;
}

export function AppUpdatePrompt({ update, open, onClose, onIgnore }: Props) {
  if (!open || !update) return null;

  async function updateNow() {
    if (!update) return;
    await openUrl(update.releaseUrl);
    onClose();
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
            <h2 id="app-update-title">New version available</h2>
            <p className="muted">FRP Manager {update.latestVersion}</p>
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

        <div className="modal-actions app-update-actions">
          <button type="button" className="command-button" onClick={onClose}>
            Later
          </button>
          <button
            type="button"
            className="command-button"
            onClick={() => onIgnore(update.latestVersion)}
          >
            Ignore this version
          </button>
          <button
            type="button"
            className="command-button primary"
            onClick={() => void updateNow()}
          >
            <Download size={16} />
            Update
          </button>
        </div>
      </section>
    </div>
  );
}
