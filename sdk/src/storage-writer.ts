//! Drains replica changes in write order, including work a previous save failed to keep.

import type { Replica } from "./replica";
import {
	StorageConflictError,
	type Storage,
	type StorageRevision,
	type StoredState,
} from "./storage";

export class StorageWriter {
	private saving: Promise<void> = Promise.resolve();
	private revision: StorageRevision = 0;
	private conflict: StorageConflictError | null = null;

	constructor(
		private readonly replica: Replica,
		private readonly storage: Storage,
	) {}

	load(state: StoredState): void {
		this.replica.load(state);
		this.revision = state.revision ?? 0;
	}

	save(): void {
		this.saving = this.saving.then(async () => {
			if (this.conflict) return;
			const change = this.replica.takeChanges();
			if (!change) return;
			try {
				await this.storage.save({ ...change, revision: this.revision });
				this.revision += 1;
			} catch (error) {
				// A failed delta must rejoin the dirty state before its successor is taken.
				this.replica.unsaved(change);
				if (error instanceof StorageConflictError) this.conflict = error;
			}
		});
	}

	async settled(): Promise<void> {
		let saving: Promise<void>;
		do {
			saving = this.saving;
			await saving;
		} while (saving !== this.saving);
		if (this.conflict) throw this.conflict;
	}

	async reload(): Promise<void> {
		const state = (await this.storage.load()) ?? { cursor: 0, rows: [], outbox: [] };
		this.replica.rebase(state);
		this.revision = state.revision ?? 0;
		this.conflict = null;
		this.save();
	}
}
