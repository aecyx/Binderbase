// SPDX-License-Identifier: AGPL-3.0-or-later
import { useEffect, useState } from "react";
import type { ReactElement } from "react";
import { TopNav } from "./components/TopNav";
import type { Route } from "./components/TopNav";
import { AboutModal } from "./components/AboutModal";
import { ScanPage } from "./features/scan/ScanPage";
import { CollectionPage } from "./features/collection/CollectionPage";
import { PricesPage } from "./features/pricing/PricesPage";
import { ImportExportPage } from "./features/import_export/ImportExportPage";
import { api } from "./lib/tauri";
import type { AppInfo, ImportStatus } from "./types";
import "./App.css";

function App(): ReactElement {
  const [route, setRoute] = useState<Route>("scan");
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [catalogEmpty, setCatalogEmpty] = useState(false);

  useEffect(() => {
    let cancelled = false;
    api
      .appInfo()
      .then((v) => {
        if (!cancelled) setInfo(v);
      })
      .catch(() => {
        // Non-fatal; the nav just won't show a version chip.
      });

    // Detect first-run: if no catalog has ever been imported, nudge the user.
    api.catalog
      .importStatus()
      .then((status: ImportStatus) => {
        if (!cancelled && !status.last_mtg && !status.last_pokemon) {
          setCatalogEmpty(true);
          setRoute("import_export");
        }
      })
      .catch(() => {
        // Non-fatal.
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="app">
      <TopNav current={route} onChange={setRoute} version={info?.version} />
      <main className="app__main" role="main">
        {info?.keyring_degraded && (
          <p className="notice" role="status">
            ⚠ OS keyring unavailable — API keys entered in Settings will not persist across
            launches.
          </p>
        )}
        {catalogEmpty && route === "import_export" && (
          <div className="onboarding-banner">
            <h2>Welcome to Binderbase!</h2>
            <p>
              To get started, import a card catalog below. This downloads card data so you can scan,
              search, and track your collection offline.
            </p>
          </div>
        )}
        {route === "scan" && <ScanPage />}
        {route === "collection" && <CollectionPage />}
        {route === "pricing" && <PricesPage />}
        {route === "import_export" && <ImportExportPage />}
      </main>
      {info && (
        <footer className="app__footer">
          <span className="muted">
            {info.name} · v{info.version} · local data: <code>{info.db_path}</code> ·{" "}
            <AboutModal version={info.version} />
          </span>
        </footer>
      )}
    </div>
  );
}

export default App;
