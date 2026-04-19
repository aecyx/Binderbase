// SPDX-License-Identifier: AGPL-3.0-or-later

#![no_main]

use libfuzzer_sys::fuzz_target;
use rusqlite::Connection;

// Fuzz the CSV import-preview parser with arbitrary byte strings.
//
// The function under test parses untrusted CSV text from user file uploads
// and should never panic regardless of input.
fuzz_target!(|data: &[u8]| {
    // Only valid UTF-8 reaches import_preview in production, but we still
    // convert lossy here to exercise edge cases.
    let text = String::from_utf8_lossy(data);

    // Minimal in-memory DB — import_preview needs a connection for FK checks
    // but we don't need any seed data; the fuzzer just exercises the parser.
    let conn = Connection::open_in_memory().unwrap();

    // We don't care about the result — only that it doesn't panic.
    let _ = binderbase_lib::collection::csv::import_preview(&conn, &text);
});
