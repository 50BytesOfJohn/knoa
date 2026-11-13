//! Drains replica changes in write order, including work a previous save failed to keep.

import type { Replica } from "./replica";
import type { Storage } from "./storage";

export class StorageWriter {
	private saving: Promise<void> = Promise.resolve();

	constructor(
		private readonly replica: Replica,
		private readonly storage: Storage,
	) {}

	save(): void {
		this.saving = this.saving.then(async () => {
			const change = this.replica.takeChanges();
			if (!change) return;
			try {
				await this.storage.save(change);
			} catch {
				// A failed delta must rejoin the dirty state before its successor is taken.
				this.replica.unsaved(change);
			}
		});
	}

	async settled(): Promise<void> {
		let saving: Promise<void>;
		do {
			saving = this.saving;
			await saving;
		} while (saving !== this.saving);
	}
}
