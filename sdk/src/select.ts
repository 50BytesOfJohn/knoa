//! What the UI asks of the library: places, search, kinds and date groups.

import type {
	Bookmark,
	BookmarkId,
	BookmarkTag,
	Folder,
	FolderId,
	WorkspaceId,
} from "./protocol";
import { FolderTree } from "./folder-tree";
import type { Library } from "./library";

export {
	type FolderNode,
	FolderTree,
	MAX_FOLDER_DEPTH,
	type MoveError,
} from "./folder-tree";

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
	const folders = (library.folders.get(id) ?? [])
		.filter((link) => !link.excluded)
		.flatMap((link) => library.folder.get(link.folder_id) ?? []);
	const [first] = folders;
	if (!first) return [];
	const order = sidebarOrder(library, first.workspace_id);
	return folders.sort((a, b) => (order.get(a.id) ?? 0) - (order.get(b.id) ?? 0));
}

/** A workspace's folders in sidebar order: each followed by its subfolders. */
export function foldersIn(library: Library, workspace: WorkspaceId): Folder[] {
	return folderTree(library, workspace).folders;
}

const trees = new WeakMap<Library, Map<WorkspaceId, FolderTree>>();

/** The workspace's folder tree, built once per library. */
export function folderTree(library: Library, workspace: WorkspaceId): FolderTree {
	let built = trees.get(library);
	if (!built) {
		built = new Map();
		trees.set(library, built);
	}
	let tree = built.get(workspace);
	if (!tree) {
		const folders = [...library.folder.values()].filter(
			(folder) => folder.workspace_id === workspace,
		);
		tree = new FolderTree(folders);
		built.set(workspace, tree);
	}
	return tree;
}

function sidebarOrder(
	library: Library,
	workspace: WorkspaceId,
): Map<FolderId, number> {
	const order = new Map<FolderId, number>();
	folderTree(library, workspace).nodes.forEach((node, at) =>
		order.set(node.folder.id, at),
	);
	return order;
}

/** The folder and every folder under it: what selecting it shows. */
function folderScope(library: Library, id: FolderId): Set<FolderId> {
	const folder = library.folder.get(id);
	if (!folder) return new Set();
	const subtree = folderTree(library, folder.workspace_id).subtree(id);
	return new Set(subtree.map((f) => f.id));
}

/** A workspace's bookmarks in a place, newest first, as the desktop sorts them. */
export function bookmarksIn(
	library: Library,
	workspace: WorkspaceId,
	place: Place,
	filter: Filter = {},
): Bookmark[] {
	const words = searchWords(filter.query ?? "");
	const scope = place.type === "folder" ? folderScope(library, place.id) : undefined;
	return [...library.bookmark.values()]
		.filter(
			(b) =>
				b.workspace_id === workspace &&
				inPlace(library, b, place, scope) &&
				(!filter.kind || b.kind === filter.kind) &&
				matches(library, b, words),
		)
		.sort((a, b) => b.added_at - a.added_at || b.id.localeCompare(a.id));
}

/** A folder place takes in its subfolders; `scope` saves working them out per bookmark. */
export function inPlace(
	library: Library,
	b: Bookmark,
	place: Place,
	scope?: Set<FolderId>,
): boolean {
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
		case "folder": {
			const folders = scope ?? folderScope(library, place.id);
			return linkedFolders(library, b.id).some((id) => folders.has(id));
		}
	}
}

export type PlaceCounts = Record<Exclude<Place["type"], "folder">, number> & {
	/** Bookmarks in each folder or under it, each counted once. */
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
	const tree = folderTree(library, workspace);
	const ancestry = new Map(
		tree.nodes.map((node) => [node.folder.id, tree.path(node.folder.id)]),
	);
	for (const b of library.bookmark.values()) {
		if (b.workspace_id !== workspace) continue;
		if (b.deleted_at !== null) counts.trash += 1;
		else if (b.set_aside !== null) counts.set_aside += 1;
		else {
			counts.everything += 1;
			if (b.up_next) counts.up_next += 1;
			if (b.status !== "organized") counts.not_organized += 1;
			const folders = linkedFolders(library, b.id);
			if (folders.length === 0) counts.unsorted += 1;
			const holding = new Set(
				folders.flatMap((id) => ancestry.get(id) ?? []).map((f) => f.id),
			);
			for (const id of holding) {
				counts.folders.set(id, (counts.folders.get(id) ?? 0) + 1);
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

/** The folders a bookmark is in, unordered. */
function linkedFolders(library: Library, id: BookmarkId): FolderId[] {
	return (library.folders.get(id) ?? [])
		.filter((link) => !link.excluded && library.folder.has(link.folder_id))
		.map((link) => link.folder_id);
}
