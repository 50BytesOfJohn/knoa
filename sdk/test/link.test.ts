import { describe, expect, it } from "vitest";
import { bookmarkIdFor, parseLink, siteOf } from "../src/link";

const WORKSPACE = "6f1c2a7e-9a51-4b8e-a3b4-1f9e2d6c8a10";

describe("parseLink", () => {
	it("turns typed links into web URLs, like the desktop", () => {
		expect(parseLink(" example.com/a ")).toBe("https://example.com/a");
		expect(parseLink("http://x.dev")).toBe("http://x.dev/");
		expect(parseLink("not a link")).toBeNull();
		expect(parseLink("mailto:me@x.dev")).toBeNull();
		expect(parseLink("notes")).toBeNull();
		expect(parseLink("ftp://example.com/file")).toBeNull();
		expect(parseLink("localhost:3000")).toBe("https://localhost:3000/");
	});

	it("names the site without www", () => {
		expect(siteOf("https://www.example.com/a")).toBe("example.com");
		expect(siteOf("https://docs.example.com/a")).toBe("docs.example.com");
	});
});

describe("bookmarkIdFor", () => {
	// Reference values from Python's uuid.uuid5, which hashes the UTF-8 bytes.
	it("is UUID v5 of the URL in the workspace's namespace", () => {
		expect(bookmarkIdFor(WORKSPACE, "https://example.com/a")).toBe(
			"17f11e50-56a4-5a6a-9e53-7320e3c06646",
		);
		expect(bookmarkIdFor(WORKSPACE, "https://example.com/café")).toBe(
			"1ef4b1cb-bdc1-5033-b8fa-aeb186f21327",
		);
	});

	it("differs between workspaces", () => {
		const other = "0b7d2c4e-1111-4a2b-9c3d-5e6f7a8b9c0d";
		expect(bookmarkIdFor(other, "https://example.com/a")).not.toBe(
			bookmarkIdFor(WORKSPACE, "https://example.com/a"),
		);
	});
});
