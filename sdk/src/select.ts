//! What the UI asks of the library: places, search, kinds and date groups.

import type {
	Bookmark,
	BookmarkId,
	BookmarkTag,
	Folder,
	WorkspaceId,
} from "./protocol";
import type { Library } from "./library";

export type Place =
	| { type: "everything" }
	| { type: "up_next" }
	| { type: "unsorted" }
	| { type: "not_organized" }
	| { type: "set_aside" }
	| { type: "trash" }
	| { type: "folder"; id: string };

export interface Filter {
	query?: string;
	kind?: string | null;
}

/** The tags on a bookmark, without the ones the user removed. */
export function tagsOf(library: Library, id: BookmarkId): BookmarkTag[] {
	return (library.tags.get(id) ?? [])
		.filter((tag) => !tag.excluded)
		.sort((a, b) => a.tag.localeCompare(b.tag));
}

/** The folders a bookmark is in, in sidebar order. */
export function foldersOf(library: Library, id: BookmarkId): Folder[] {
	return (library.folders.get(id) ?? [])
		.filter((link) => !link.excluded)
		.flatMap((link) => library.folder.get(link.folder_id) ?? [])
		.sort(byPosition);
}

export function foldersIn(library: Library, workspace: WorkspaceId): Folder[] {
	return [...library.folder.values()]
		.filter((folder) => folder.workspace_id === workspace)
		.sort(byPosition);
}

/** A workspace's bookmarks in a place, newest first, as the desktop sorts them. */
export function bookmarksIn(
	library: Library,
	workspace: WorkspaceId,
	place: Place,
	filter: Filter = {},
): Bookmark[] {
	const words = searchWords(filter.query ?? "");
	return [...library.bookmark.values()]
		.filter(
			(b) =>
				b.workspace_id === workspace &&
				inPlace(library, b, place) &&
				(!filter.kind || b.kind === filter.kind) &&
				matches(library, b, words),
		)
		.sort((a, b) => b.added_at - a.added_at || b.id.localeCompare(a.id));
}

export function inPlace(library: Library, b: Bookmark, place: Place): boolean {
	if (place.type === "trash") return b.deleted_at !== null;
	if (b.deleted_at !== null) return false;
	if (place.type === "set_aside") return b.set_aside !== null;
	if (b.set_aside !== null) return false;
	switch (place.type) {
		case "everything":
			return true;
		case "up_next":
			return b.up_next;
		case "unsorted":
			return foldersOf(library, b.id).length === 0;
		case "not_organized":
			return b.status !== "organized";
		case "folder":
			return foldersOf(library, b.id).some((f) => f.id === place.id);
	}
}

export type PlaceCounts = Record<Exclude<Place["type"], "folder">, number> & {
	folders: Map<string, number>;
};

export function placeCounts(
	library: Library,
	workspace: WorkspaceId,
): PlaceCounts {
	const counts: PlaceCounts = {
		everything: 0,
		up_next: 0,
		unsorted: 0,
		not_organized: 0,
		set_aside: 0,
		trash: 0,
		folders: new Map(),
	};
	for (const b of library.bookmark.values()) {
		if (b.workspace_id !== workspace) continue;
		if (b.deleted_at !== null) counts.trash += 1;
		else if (b.set_aside !== null) counts.set_aside += 1;
		else {
			counts.everything += 1;
			if (b.up_next) counts.up_next += 1;
			if (b.status !== "organized") counts.not_organized += 1;
			const folders = foldersOf(library, b.id);
			if (folders.length === 0) counts.unsorted += 1;
			for (const f of folders) {
				counts.folders.set(f.id, (counts.folders.get(f.id) ?? 0) + 1);
			}
		}
	}
	return counts;
}

/** Kinds among these bookmarks, most common first, for the kind chips. */
export function kindsAmong(
	bookmarks: Bookmark[],
): [kind: string, count: number][] {
	const counts = new Map<string, number>();
	for (const b of bookmarks) {
		if (b.kind) counts.set(b.kind, (counts.get(b.kind) ?? 0) + 1);
	}
	return [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
}

export interface DateGroup {
	label: string;
	bookmarks: Bookmark[];
}

const MONTHS = new Intl.DateTimeFormat("en", { month: "long" });

/** Today, Yesterday, This week, Last week, then months, then years, as on the desktop. */
export function groupByDate(
	bookmarks: Bookmark[],
	now = new Date(),
): DateGroup[] {
	const groups: DateGroup[] = [];
	for (const b of bookmarks) {
		const label = dateLabel(new Date(b.added_at * 1000), now);
		const last = groups.at(-1);
		if (last?.label === label) last.bookmarks.push(b);
		else groups.push({ label, bookmarks: [b] });
	}
	return groups;
}

function dateLabel(date: Date, now: Date): string {
	const today = startOfDay(now);
	const day = startOfDay(date);
	const days = Math.round((today - day) / 86_400_000);
	if (days <= 0) return "Today";
	if (days === 1) return "Yesterday";
	// Weeks start on Monday.
	const weekday = (now.getDay() + 6) % 7;
	if (days <= weekday) return "This week";
	if (days <= weekday + 7) return "Last week";
	if (date.getFullYear() === now.getFullYear()) return MONTHS.format(date);
	return String(date.getFullYear());
}

function startOfDay(date: Date): number {
	return new Date(
		date.getFullYear(),
		date.getMonth(),
		date.getDate(),
	).getTime();
}

// ── Search ─────────────────────────────────────────────────────────────────────────────

/** Every word must match, each as a prefix, so results narrow as the user types. */
function searchWords(query: string): string[] {
	return query
		.toLowerCase()
		.split(/[^\p{L}\p{N}]+/u)
		.filter((word) => word !== "");
}

/** Tokens per bookmark, valid while its row and its tag list are the same objects. */
const indexed = new WeakMap<
	Bookmark,
	{ tags: BookmarkTag[] | undefined; tokens: string[] }
>();

function matches(library: Library, b: Bookmark, words: string[]): boolean {
	if (words.length === 0) return true;
	const tags = library.tags.get(b.id);
	let entry = indexed.get(b);
	if (!entry || entry.tags !== tags) {
		const names = tagsOf(library, b.id).map((t) => t.tag);
		const text = [b.title, b.url, b.site, b.summary ?? "", ...names].join(" ");
		entry = { tags, tokens: searchWords(text) };
		indexed.set(b, entry);
	}
	const { tokens } = entry;
	return words.every((word) => tokens.some((token) => token.startsWith(word)));
}

function byPosition(a: Folder, b: Folder): number {
	return a.position - b.position || a.name.localeCompare(b.name);
}
