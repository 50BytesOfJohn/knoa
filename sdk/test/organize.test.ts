import { expect, it } from "vitest";
import { Api } from "../src/api";
import { FakeServer } from "../src/fake-server";
import { addLink, createWorkspace } from "../src/mutations";
import { type Account, PLAN_LIMITS } from "../src/protocol";
import { clientFor, started } from "./fixtures";

/** A server holding `count` bookmarks saved by one client, and the API to ask it. */
async function saved(count: number, plan: Account["plan"] = "free") {
	const server = new FakeServer({ plan });
	const client = await started(clientFor(server));
	const { id, mutation } = createWorkspace("Personal", 0);
	const keys: string[] = [];
	const saves = Array.from({ length: count }, (_, i) => {
		const added = addLink(client.getSnapshot().library, id, `example.com/${i}`);
		if (!added) throw new Error("not a link");
		keys.push(added.id);
		return added.mutations;
	}).flat();
	client.mutate([mutation, ...saves]);
	await client.settled();
	const api = new Api("http://api.test", async () => "token", server.fetch);
	return { server, api, keys };
}

it("organizes a free account's new saves in the cloud until its month's allowance is used", async () => {
	const room = PLAN_LIMITS.free.organized_per_month;
	const { server, api, keys } = await saved(room + 5);

	const account = await api.me();
	expect(account.organizer).toBe("device");
	expect(account.usage.organized_this_month).toBe(room);
	expect(server.tables.bookmark.get(keys[0])?.status).toBe("queued");
	expect(server.tables.bookmark.get(keys[room])?.status).toBe("new");
	// What doesn't fit is not remembered for a free account, and folders-only runs stop too.
	expect(
		await api.organize({ scope: { all: true }, folders_only: false }),
	).toEqual({ queued: 0, pending: 0 });
	expect(
		await api.organize({ scope: { all: true }, folders_only: true }),
	).toEqual({ queued: 0, pending: 0 });
});

it("accepts what it queued or already had, and refuses what a free month has no room for", async () => {
	const room = PLAN_LIMITS.free.organized_per_month;
	const { api, keys } = await saved(room + 1);
	const [waiting, refused] = [keys[0], keys[room]];

	const answer = await api.organize({
		scope: { bookmarks: [waiting, refused, "gone"] },
		folders_only: false,
	});

	expect(answer).toEqual({ queued: 0, pending: 0, accepted: [waiting, "gone"] });
	expect((await api.me()).usage.organized_this_month).toBe(room);
});

it("keeps a subscriber's requests beyond the month's room as pending", async () => {
	const room = PLAN_LIMITS.pro.organized_per_month;
	const { api, keys } = await saved(room + 2, "pro");
	const beyond = keys.slice(room);

	const answer = await api.organize({
		scope: { bookmarks: beyond },
		folders_only: false,
	});

	expect(answer).toEqual({ queued: 0, pending: 2, accepted: beyond });
	expect((await api.me()).organizer).toBe("cloud");
});
