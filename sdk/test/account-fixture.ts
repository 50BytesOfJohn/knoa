import fixture from "../fixtures/account-and-organize.json";
import type { Account, OrganizeResponse } from "../src/protocol";

// JSON imports widen string literals to `string`, so widen the protocol's the same way.
type Json<T> = T extends string
	? string
	: T extends object
		? { [K in keyof T]: Json<T[K]> }
		: T;

// `pnpm typecheck` fails here when the protocol gains a required field the fixture lacks,
// and the fixture's Rust round trip (crates/knoa-core/tests/protocol_fixtures.rs) then
// fails until Rust keeps that field too.
fixture satisfies {
	accounts: Json<Account>[];
	organize: Json<OrganizeResponse>[];
};
