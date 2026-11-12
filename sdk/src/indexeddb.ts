//! The replica in IndexedDB, so a reload renders the library before the network answers.

import type { Mutation } from "./protocol";
import {
	outboxSequence,
	rowId,
	type Storage,
	type StorageChange,
	type StoredRow,
	type StoredState,
} from "./storage";

const ROWS = "rows";
const META = "meta";
const OUTBOX = "outbox";
const STORES = [ROWS, META, OUTBOX];

interface Meta {
	cursor: number;
	/** Version 1 kept the whole outbox here; version 2 moves it to `outbox`. */
	outbox?: Mutation[];
}

interface OutboxEntry {
	seq: number;
	mutation: Mutation;
}

/** One database per name; name it after the user so accounts never share a replica. */
export class IndexedDbStorage implements Storage {
	private db: Promise<IDBDatabase> | null = null;
	private readonly next = outboxSequence();

	constructor(private readonly name: string) {}

	/** Deletes a replica, once every tab that has it open lets go of it. */
	static forget(name: string): Promise<void> {
		return new Promise((resolve, reject) => {
			const request = indexedDB.deleteDatabase(name);
			request.onsuccess = () => resolve();
			request.onerror = () => reject(request.error);
		});
	}

	async load(): Promise<StoredState | null> {
		const db = await this.open();
		const tx = db.transaction(STORES, "readonly");
		const [rows, meta, outbox] = await Promise.all([
			request<StoredRow[]>(tx.objectStore(ROWS).getAll()),
			request<Meta | undefined>(tx.objectStore(META).get("state")),
			request<OutboxEntry[]>(tx.objectStore(OUTBOX).getAll()),
		]);
		if (!meta) return null;
		const ordered = outbox
			.sort((a, b) => a.seq - b.seq)
			.map((entry) => entry.mutation);
		return { cursor: meta.cursor, outbox: ordered, rows };
	}

	async save(change: StorageChange): Promise<void> {
		const db = await this.open();
		const tx = db.transaction(STORES, "readwrite");
		const rows = tx.objectStore(ROWS);
		for (const row of change.put) rows.put(row, rowId(row));
		for (const row of change.remove) rows.delete(rowId(row));
		const outbox = tx.objectStore(OUTBOX);
		for (const mutation of change.enqueue) {
			const entry: OutboxEntry = { seq: this.next(), mutation };
			outbox.put(entry, mutation.mid);
		}
		for (const mid of change.dequeue) outbox.delete(mid);
		const meta: Meta = { cursor: change.cursor };
		tx.objectStore(META).put(meta, "state");
		await done(tx);
	}

	async clear(): Promise<void> {
		const db = await this.open();
		const tx = db.transaction(STORES, "readwrite");
		for (const store of STORES) tx.objectStore(store).clear();
		await done(tx);
	}

	private open(): Promise<IDBDatabase> {
		this.db ??= new Promise((resolve, reject) => {
			const open = indexedDB.open(this.name, 2);
			open.onupgradeneeded = (event) => {
				const db = open.result;
				if (event.oldVersion < 1) {
					db.createObjectStore(ROWS);
					db.createObjectStore(META);
				}
				db.createObjectStore(OUTBOX);
				if (event.oldVersion === 1) moveOutbox(open.transaction);
			};
			open.onsuccess = () => {
				// Another tab deleting the replica on sign-out waits for this to let go.
				open.result.onversionchange = () => open.result.close();
				resolve(open.result);
			};
			open.onerror = () => reject(open.error);
		});
		return this.db;
	}
}

/** Version 1's outbox array becomes one entry per mutation, in the same order. */
function moveOutbox(tx: IDBTransaction | null): void {
	if (!tx) return;
	const metas = tx.objectStore(META);
	const read = metas.get("state");
	read.onsuccess = () => {
		const meta = read.result as Meta | undefined;
		if (!meta?.outbox) return;
		const outbox = tx.objectStore(OUTBOX);
		meta.outbox.forEach((mutation, seq) => {
			outbox.put({ seq, mutation } satisfies OutboxEntry, mutation.mid);
		});
		metas.put({ cursor: meta.cursor } satisfies Meta, "state");
	};
}

function request<T>(req: IDBRequest): Promise<T> {
	return new Promise((resolve, reject) => {
		req.onsuccess = () => resolve(req.result as T);
		req.onerror = () => reject(req.error);
	});
}

function done(tx: IDBTransaction): Promise<void> {
	return new Promise((resolve, reject) => {
		tx.oncomplete = () => resolve();
		tx.onerror = () => reject(tx.error);
		tx.onabort = () => reject(tx.error);
	});
}
