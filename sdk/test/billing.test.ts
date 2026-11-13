import { expect, it } from "vitest";
import { Api, HttpError } from "../src/api";
import { FakeServer } from "../src/fake-server";
import { BILLING_ERROR } from "../src/protocol";

const apiFor = (server: FakeServer) =>
	new Api("http://api.test", async () => "token", server.fetch);

it("checks out to Pro once, then answers already_subscribed", async () => {
	const server = new FakeServer();
	const api = apiFor(server);

	expect(await api.checkout({ period: "annual" })).toEqual({
		url: "/app?checkout=success",
	});
	expect(await api.refreshBilling()).toEqual({ plan: "pro" });
	await expect(api.checkout({ period: "month" })).rejects.toMatchObject({
		status: 409,
		code: BILLING_ERROR.alreadySubscribed,
	});
});

it("has no portal for an account that never had Pro", async () => {
	const api = apiFor(new FakeServer());
	const error = await api.portal().catch((e: unknown) => e);
	expect(error).toBeInstanceOf(HttpError);
	expect(error).toMatchObject({
		status: 404,
		code: BILLING_ERROR.noSubscription,
	});
});
