import { useEffect, useState } from "react";
import type { ReactElement } from "react";
import { TopNav } from "./components/TopNav";
import type { Route } from "./components/TopNav";
import { ScanPage } from "./features/scan/ScanPage";
import { CollectionPage } from "./features/collection/CollectionPage";
import { PricesPage } from "./features/pricing/PricesPage";
import { ImportExportPage } from "./features/import_export/ImportExportPage";
import { api } from "./lib/tauri";
import type { AppInfo } from "./types";
import "./App.css";

function App(): ReactElement {
  const [route, setRoute] = useState<Route>("scan");
  const [info, setInfo] = useState<AppInfo | null>(null);

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
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="app">
      <TopNav current={route} onChange={setRoute} version={info?.version} />
      <main className="app__main" role="main">
        {route === "scan" && <ScanPage />}
        {route === "collection" && <CollectionPage />}
        {route === "pricing" && <PricesPage />}
        {route === "import_export" && <ImportExportPage />}
      </main>
      {info && (
        <footer className="app__footer">
          <span className="muted">
            {info.name} · v{info.version} · local data: <code>{info.db_path}</code>
          </span>
        </footer>
      )}
    </div>
  );
}

export default App;
