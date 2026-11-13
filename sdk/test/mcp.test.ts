import { expect, it } from "vitest";
import fixture from "../fixtures/mcp-settings.json";
import { Api } from "../src/api";
import type { McpSettings, McpSettingsResponse } from "../src/protocol";

// JSON imports widen string literals to `string`, as in account-fixture.ts.
type Json<T> = T extends string
	? string
	: T extends object
		? { [K in keyof T]: Json<T[K]> }
		: T;

Object.values(fixture.settings) satisfies Json<McpSettings>[];
({
	...fixture.settings.default,
	url: fixture.url,
}) satisfies Json<McpSettingsResponse>;

it("reads MCP settings and replaces them, including clearing workspace exceptions", async () => {
	let stored = fixture.settings.default;
	const api = new Api("http://api.test", async () => "token", async (url, init) => {
		expect(url).toBe("http://api.test/v1/mcp/settings");
		const headers = new Headers(init?.headers);
		expect(headers.get("Authorization")).toBe("Bearer token");
		if (init?.method === "PUT") {
			expect(headers.get("Content-Type")).toBe("application/json");
			stored = JSON.parse(init.body as string);
		} else {
			expect(init?.method).toBe("GET");
			expect(init?.body).toBeUndefined();
		}
		return Response.json({ ...stored, url: fixture.url });
	});

	const initial: McpSettingsResponse = await api.mcpSettings();
	expect(initial).toEqual({ ...fixture.settings.default, url: fixture.url });
	for (const settings of [fixture.settings.overrides, fixture.settings.off]) {
		const body = settings as McpSettings;
		expect(await api.setMcpSettings(body)).toEqual({
			...settings,
			url: fixture.url,
		});
	}
});
