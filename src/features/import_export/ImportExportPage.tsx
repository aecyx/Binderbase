// SPDX-License-Identifier: AGPL-3.0-or-later
import type { ReactElement } from "react";
import { CatalogImportPanel } from "./CatalogImportPanel";
import { ScanIndexPanel } from "./ScanIndexPanel";

export function ImportExportPage(): ReactElement {
  return (
    <section aria-labelledby="io-heading">
      <h1 id="io-heading">Import / Export</h1>

      <CatalogImportPanel />

      <hr />

      <ScanIndexPanel />

      <hr />

      <h2>Collection Import / Export</h2>
      <p className="muted">
        CSV collection import/export and deck-list import/export (Moxfield-style for MTG,
        PTCGL-style for Pokémon) are planned for 1.0. Not yet wired.
      </p>
      <ul>
        <li>Import a CSV into your collection (dry-run + summary before commit).</li>
        <li>Export your collection as CSV or a plain deck list.</li>
        <li>Import a deck list and reconcile it against what you own.</li>
      </ul>
    </section>
  );
}
