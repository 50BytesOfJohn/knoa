//! Cutting the outbox into pushes the server takes.

import { MAX_PUSH, MAX_PUSH_BYTES, type Mutation } from "./protocol";

const encoder = new TextEncoder();
/** Room for `{"mutations":[...]}` and the commas. */
const ENVELOPE_BYTES = 1024;

/** The next push: in order, at most `MAX_PUSH` mutations and `MAX_PUSH_BYTES`, never empty. */
export function nextBatch(pending: readonly Mutation[]): Mutation[] {
	const batch: Mutation[] = [];
	let bytes = ENVELOPE_BYTES;
	for (const mutation of pending) {
		if (batch.length === MAX_PUSH) break;
		const size = encoder.encode(JSON.stringify(mutation)).byteLength + 1;
		if (batch.length > 0 && bytes + size > MAX_PUSH_BYTES) break;
		batch.push(mutation);
		bytes += size;
	}
	return batch;
}
