import type { ReactElement } from "react";

export type Route = "scan" | "collection" | "pricing" | "import_export";

const ROUTES: { id: Route; label: string }[] = [
  { id: "scan", label: "Scan" },
  { id: "collection", label: "Collection" },
  { id: "pricing", label: "Prices" },
  { id: "import_export", label: "Import / Export" },
];

interface Props {
  current: Route;
  onChange: (route: Route) => void;
  version?: string;
}

export function TopNav({ current, onChange, version }: Props): ReactElement {
  return (
    <header className="topnav" role="banner">
      <div className="topnav__brand">
        <span className="topnav__mark" aria-hidden>
          ◆
        </span>
        <strong>Binderbase</strong>
        {version && <span className="topnav__version">v{version}</span>}
      </div>
      <nav aria-label="Primary">
        <ul className="topnav__tabs">
          {ROUTES.map((r) => (
            <li key={r.id}>
              <button
                type="button"
                aria-current={current === r.id ? "page" : undefined}
                onClick={() => onChange(r.id)}
              >
                {r.label}
              </button>
            </li>
          ))}
        </ul>
      </nav>
    </header>
  );
}
