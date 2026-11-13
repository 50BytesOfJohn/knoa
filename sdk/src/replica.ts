//! The local copy of the library: what the server has confirmed, plus the outbox on top.

import type {
	Change,
	Entity,
	Key,
	Mutation,
	MutationResult,
	RejectReason,
} from "./protocol";
import { buildLibrary, type Library } from "./library";
import type { StorageChange, StoredState } from "./storage";
import {
	applyMutation,
	copyTables,
	deleteRow,
	emptyTables,
	putRow,
	type Tables,
} from "./tables";

/** A mutation the server turned down, with what it was. */
export interface Rejection {
	mutation: Mutation;
	reason: RejectReason | "ignored";
}

/**
 * The confirmed rows (`base`) only change from pulls and from pushes the server applied.
 * Local edits live in the outbox and are replayed over `base` to make the view, so a pull
 * never overwrites a column that still has a pending change, and dropping a mutation from
 * the outbox is all it takes to roll it back.
 */
export class Replica {
	private base: Tables = emptyTables();
	private outbox: Mutation[] = [];
	private cursorValue = 0;
	private dirtyRows = new Map<string, { entity: Entity; key: Key }>();
	private dirtyMeta = false;
	private enqueued: Mutation[] = [];
	private dequeued = new Set<string>();
	private cached: Library | null = null;

	get cursor(): number {
		return this.cursorValue;
	}

	get pending(): readonly Mutation[] {
		return this.outbox;
	}

	load(state: StoredState): void {
		this.base = emptyTables();
		for (const row of state.rows) {
			(this.base[row.entity] as Map<Key, unknown>).set(row.key, row.data);
		}
		this.outbox = [...state.outbox];
		this.cursorValue = state.cursor;
		this.dirtyRows.clear();
		this.dirtyMeta = false;
		this.enqueued = [];
		this.dequeued.clear();
		this.cached = null;
	}

	/** Replaces stale confirmed rows while retaining edits this client has yet to save. */
	rebase(state: StoredState): void {
		const pending = new Set(this.outbox.map((mutation) => mutation.mid));
		const enqueued = this.enqueued.filter((mutation) => pending.has(mutation.mid));
		const dequeued = new Set(this.dequeued);
		this.load(state);
		if (dequeued.size > 0) this.discard(dequeued);
		const stored = new Set(this.outbox.map((mutation) => mutation.mid));
		// A saved entry missing from the new snapshot was answered by another tab.
		this.enqueue(enqueued.filter((mutation) => !stored.has(mutation.mid)));
	}

	/** The library as the user sees it: confirmed rows with pending edits applied. */
	library(): Library {
		if (!this.cached) {
			const view = this.outbox.length > 0 ? copyTables(this.base) : this.base;
			for (const mutation of this.outbox) applyMutation(view, mutation);
			this.cached = buildLibrary(view);
		}
		return this.cached;
	}

	enqueue(mutations: Mutation[]): void {
		if (mutations.length === 0) return;
		this.outbox.push(...mutations);
		this.enqueued.push(...mutations);
		this.dirtyMeta = true;
		this.cached = null;
	}

	/** Applies a page of pulled changes, in any order of parents and children. */
	applyPull(changes: Change[], cursor: number): void {
		for (const change of changes) {
			if (change.deleted) {
				this.markRemoved(deleteRow(this.base, change.entity, change.key));
			} else {
				// A pulled row is the whole row, so it replaces what was here.
				this.base[change.entity].delete(change.key);
				putRow(this.base, change.entity, change.key, change.data);
				this.markDirty(change.entity, change.key);
			}
		}
		this.cursorValue = Math.max(this.cursorValue, cursor);
		this.dirtyMeta = true;
		this.cached = null;
	}

	/**
	 * Takes the server's answer to a push. Applied mutations move into the confirmed rows
	 * so nothing flickers before the next pull; the rest are dropped, which rolls them back.
	 */
	settle(results: MutationResult[]): Rejection[] {
		const byMid = new Map(results.map((result) => [result.mid, result]));
		const rejections: Rejection[] = [];
		const kept: Mutation[] = [];
		for (const mutation of this.outbox) {
			const result = byMid.get(mutation.mid);
			if (!result) {
				kept.push(mutation);
			} else if (result.status === "applied") {
				this.confirm(mutation);
			} else {
				rejections.push({
					mutation,
					reason:
						result.status === "ignored"
							? "ignored"
							: (result.reason ?? "invalid"),
				});
			}
		}
		this.markDequeued(this.outbox, kept);
		this.outbox = kept;
		this.dirtyMeta = true;
		this.cached = null;
		return rejections;
	}

	/** Drops mutations without an answer from the server: it refused the whole request. */
	discard(mids: Set<string>): Mutation[] {
		const dropped = this.outbox.filter((m) => mids.has(m.mid));
		const kept = this.outbox.filter((m) => !mids.has(m.mid));
		this.markDequeued(this.outbox, kept);
		this.outbox = kept;
		this.dirtyMeta = true;
		this.cached = null;
		return dropped;
	}

	/** What changed since the last call, for the storage to write. */
	takeChanges(): StorageChange | null {
		if (this.dirtyRows.size === 0 && !this.dirtyMeta) return null;
		const put: StorageChange["put"] = [];
		const remove: StorageChange["remove"] = [];
		for (const { entity, key } of this.dirtyRows.values()) {
			const data = this.base[entity].get(key);
			if (data) put.push({ entity, key, data });
			else remove.push({ entity, key });
		}
		const change: StorageChange = {
			cursor: this.cursorValue,
			put,
			remove,
			enqueue: this.enqueued,
			dequeue: [...this.dequeued],
		};
		this.dirtyRows.clear();
		this.dirtyMeta = false;
		this.enqueued = [];
		this.dequeued = new Set();
		return change;
	}

	/** A change storage failed to write: it goes out again with the next one. */
	unsaved(change: StorageChange): void {
		for (const row of [...change.put, ...change.remove]) {
			const id = `${row.entity}\u0000${row.key}`;
			if (!this.dirtyRows.has(id))
				this.dirtyRows.set(id, { entity: row.entity, key: row.key });
		}
		this.enqueued = [...change.enqueue, ...this.enqueued];
		for (const mid of change.dequeue) this.dequeued.add(mid);
		this.dirtyMeta = true;
	}

	private markDequeued(before: Mutation[], after: Mutation[]): void {
		const left = new Set(after.map((m) => m.mid));
		for (const mutation of before) {
			if (!left.has(mutation.mid)) this.dequeued.add(mutation.mid);
		}
	}

	private confirm(mutation: Mutation): void {
		if (mutation.op === "delete") {
			this.markRemoved(deleteRow(this.base, mutation.entity, mutation.key));
		} else {
			putRow(this.base, mutation.entity, mutation.key, mutation.data);
			this.markDirty(mutation.entity, mutation.key);
		}
	}

	private markRemoved(removed: { entity: Entity; key: Key }[]): void {
		for (const { entity, key } of removed) this.markDirty(entity, key);
	}

	private markDirty(entity: Entity, key: Key): void {
		this.dirtyRows.set(`${entity}\u0000${key}`, { entity, key });
	}
}
