//! Where the replica keeps itself between sessions: IndexedDB in browsers, memory in tests,
//! SQLite on phones later.

import type { Entity, Key, Mutation, Rows } from "./protocol";

export interface StoredRow {
	entity: Entity;
	key: Key;
	data: Rows[Entity];
}

export interface StoredState {
	cursor: number;
	rows: StoredRow[];
	/** In the order the mutations were made. */
	outbox: Mutation[];
}

/**
 * One write: rows to put or remove, the cursor as it is now, and the outbox's changes.
 * The outbox is kept one mutation at a time, so two tabs sharing a database add and
 * remove their own mutations instead of overwriting each other's.
 */
export interface StorageChange {
	cursor: number;
	put: StoredRow[];
	remove: { entity: Entity; key: Key }[];
	enqueue: Mutation[];
	/** Mids the server answered; applied after `enqueue`. */
	dequeue: string[];
}

export interface Storage {
	/** `null` when nothing was stored yet. */
	load(): Promise<StoredState | null>;
	/** Writes are applied in the order they are made. */
	save(change: StorageChange): Promise<void>;
	clear(): Promise<void>;
}

/** Orders outbox entries across tabs by when they were stored. */
export function outboxSequence(): () => number {
	let last = 0;
	return () => {
		last = Math.max(last + 1, Date.now() * 1000);
		return last;
	};
}

export class MemoryStorage implements Storage {
	private cursor: number | null = null;
	private rows = new Map<string, StoredRow>();
	private outbox = new Map<string, { seq: number; mutation: Mutation }>();
	private readonly next = outboxSequence();

	async load(): Promise<StoredState | null> {
		if (this.cursor === null) return null;
		const outbox = [...this.outbox.values()]
			.sort((a, b) => a.seq - b.seq)
			.map((entry) => entry.mutation);
		return { cursor: this.cursor, rows: [...this.rows.values()], outbox };
	}

	async save(change: StorageChange): Promise<void> {
		for (const row of change.put) this.rows.set(rowId(row), row);
		for (const row of change.remove) this.rows.delete(rowId(row));
		for (const mutation of change.enqueue) {
			this.outbox.set(mutation.mid, { seq: this.next(), mutation });
		}
		for (const mid of change.dequeue) this.outbox.delete(mid);
		this.cursor = change.cursor;
	}

	async clear(): Promise<void> {
		this.cursor = null;
		this.rows.clear();
		this.outbox.clear();
	}
}

export const rowId = (row: { entity: Entity; key: Key }) =>
	`${row.entity}\u0000${row.key}`;
