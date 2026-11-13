//! The replica in IndexedDB, so a reload renders the library before the network answers.

import type { Mutation } from "./protocol";
import {
	outboxSequence,
	rowId,
	StorageConflictError,
	type Storage,
	type StorageChange,
	type StorageRevision,
	type StoredRow,
	type StoredState,
} from "./storage";

const ROWS = "rows";
const META = "meta";
const OUTBOX = "outbox";
const STORES = [ROWS, META, OUTBOX];

interface Meta {
	cursor: number;
	revision?: StorageRevision;
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
			done(tx),
		]);
		if (!meta) return null;
		const ordered = outbox
			.sort((a, b) => a.seq - b.seq)
			.map((entry) => entry.mutation);
		return {
			revision: meta.revision ?? 0,
			cursor: meta.cursor,
			outbox: ordered,
			rows,
		};
	}

	async save(change: StorageChange): Promise<void> {
		const db = await this.open();
		const tx = db.transaction(STORES, "readwrite");
		const completed = done(tx);
		let failure: unknown;
		const read = tx.objectStore(META).get("state");
		// The comparison and every write share a transaction, including across connections.
		read.onsuccess = () => {
			try {
				const meta = read.result as Meta | undefined;
				const revision = meta?.revision ?? 0;
				if (change.revision !== undefined && change.revision !== revision) {
					throw new StorageConflictError();
				}
				writeChange(tx, change, revision + 1, this.next);
			} catch (error) {
				failure = error;
				tx.abort();
			}
		};
		await completed.catch((error: unknown) => {
			throw failure ?? error;
		});
	}

	async clear(): Promise<void> {
		const db = await this.open();
		const tx = db.transaction(STORES, "readwrite");
		for (const store of STORES) tx.objectStore(store).clear();
		await done(tx);
	}

	private open(): Promise<IDBDatabase> {
		this.db ??= new Promise((resolve, reject) => {
			const open = indexedDB.open(this.name, 3);
			open.onupgradeneeded = (event) => {
				const db = open.result;
				if (event.oldVersion < 1) {
					db.createObjectStore(ROWS);
					db.createObjectStore(META);
				}
				if (event.oldVersion < 2) db.createObjectStore(OUTBOX);
				upgradeMeta(open.transaction, event.oldVersion);
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

function writeChange(
	tx: IDBTransaction,
	change: StorageChange,
	revision: StorageRevision,
	next: () => number,
): void {
	const rows = tx.objectStore(ROWS);
	for (const row of change.put) rows.put(row, rowId(row));
	for (const row of change.remove) rows.delete(rowId(row));
	const outbox = tx.objectStore(OUTBOX);
	for (const mutation of change.enqueue) {
		outbox.put({ seq: next(), mutation } satisfies OutboxEntry, mutation.mid);
	}
	for (const mid of change.dequeue) outbox.delete(mid);
	tx.objectStore(META).put(
		{ cursor: change.cursor, revision } satisfies Meta,
		"state",
	);
}

function upgradeMeta(tx: IDBTransaction | null, oldVersion: number): void {
	if (!tx) return;
	const metas = tx.objectStore(META);
	const read = metas.get("state");
	read.onsuccess = () => {
		const meta = read.result as Meta | undefined;
		if (!meta) return;
		if (oldVersion === 1) {
			const outbox = tx.objectStore(OUTBOX);
			meta.outbox?.forEach((mutation, seq) => {
				outbox.put({ seq, mutation } satisfies OutboxEntry, mutation.mid);
			});
		}
		metas.put({ cursor: meta.cursor, revision: 0 } satisfies Meta, "state");
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
		tx.onabort = () => reject(tx.error);
	});
}
