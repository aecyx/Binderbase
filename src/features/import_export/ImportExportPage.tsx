// SPDX-License-Identifier: AGPL-3.0-or-later
import type { ReactElement } from "react";
import { CatalogImportPanel } from "./CatalogImportPanel";
import { CollectionCsvPanel } from "./CollectionCsvPanel";
import { ScanIndexPanel } from "./ScanIndexPanel";

export function ImportExportPage(): ReactElement {
  return (
    <section aria-labelledby="io-heading">
      <h1 id="io-heading">Import / Export</h1>

      <CatalogImportPanel />

      <hr />

      <ScanIndexPanel />

      <hr />

      <CollectionCsvPanel />
    </section>
  );
}
