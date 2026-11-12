import type { Change, Mutation } from "../src/protocol";
import { SyncClient, type SyncOptions } from "../src/client";
import type { FakeServer } from "../src/fake-server";
import { createWorkspace } from "../src/mutations";
import { MemoryStorage, type Storage } from "../src/storage";

export function clientFor(
	server: FakeServer,
	storage: Storage = new MemoryStorage(),
	socket: SyncOptions["socket"] = server.socket,
) {
	return new SyncClient({
		apiUrl: "http://api.test",
		getToken: async () => "token",
		storage,
		fetch: server.fetch,
		socket,
		watchBrowser: false,
	});
}

/** Starts a client and waits for its first sync. */
export async function started(client: SyncClient) {
	await client.start();
	await client.settled();
	return client;
}

export function workspaceChange(id: string, version: number): Change {
	const { mutation } = createWorkspace("Personal", 0);
	const data = { ...(mutation as Extract<Mutation, { op: "put" }>).data, id };
	return {
		entity: "workspace",
		key: id,
		version,
		deleted: false,
		data,
	} as Change;
}
