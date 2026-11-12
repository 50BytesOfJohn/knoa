import { readFileSync } from "node:fs";
import { strictEqual } from "node:assert";

strictEqual(
  readFileSync(new URL("../src/kinds.json", import.meta.url), "utf8"),
  readFileSync(new URL("../../crates/knoa-core/data/kinds.json", import.meta.url), "utf8"),
  "Copy crates/knoa-core/data/kinds.json to sdk/src/kinds.json after changing kinds",
);
