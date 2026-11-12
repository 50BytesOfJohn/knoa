import type { BookmarkId, WorkspaceId } from "./protocol";
import { v5 as uuidv5 } from "uuid";

/**
 * Turns what the user typed or pasted into a web URL, adding `https://` when the scheme is
 * missing. `null` when it isn't a web address. A port of the desktop's `parse_link`.
 */
export function parseLink(input: string): string | null {
	const text = input.trim();
	if (text === "" || /\s/u.test(text)) return null;
	let url: URL;
	try {
		url = new URL(text.includes("://") ? text : `https://${text}`);
	} catch {
		return null;
	}
	const web = url.protocol === "http:" || url.protocol === "https:";
	// Credentials in a link are almost always a paste mistake (`mailto:` read as a user).
	const credentials = url.username !== "" || url.password !== "";
	const host = url.hostname;
	const real = host.includes(".") || host === "localhost";
	return web && !credentials && real ? url.href : null;
}

/** The host without `www.`, which is how sites are shown and filtered. */
export function siteOf(url: string): string {
	try {
		return new URL(url).hostname.replace(/^(www\.)+/u, "");
	} catch {
		return "";
	}
}

/** Two devices that save the same URL in the same workspace make the same row. */
export function bookmarkIdFor(workspace: WorkspaceId, url: string): BookmarkId {
	return uuidv5(url, workspace);
}

/** Whether a stored url is safe to link to: only http(s), never `javascript:` and the like. */
export function isWebUrl(url: string): boolean {
	try {
		const { protocol } = new URL(url);
		return protocol === "http:" || protocol === "https:";
	} catch {
		return false;
	}
}
