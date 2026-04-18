import type { ReactElement } from "react";

export function ImportExportPage(): ReactElement {
  return (
    <section aria-labelledby="io-heading">
      <h1 id="io-heading">Import / Export</h1>
      <p className="muted">
        CSV collection import/export and deck-list import/export (Moxfield-style
        for MTG, PTCGL-style for Pokémon) are planned for 1.0. Not yet wired.
      </p>
      <p>
        <strong>Coming:</strong>
      </p>
      <ul>
        <li>Import a CSV into your collection (dry-run + summary before commit).</li>
        <li>Export your collection as CSV or a plain deck list.</li>
        <li>Import a deck list and reconcile it against what you own.</li>
      </ul>
    </section>
  );
}
