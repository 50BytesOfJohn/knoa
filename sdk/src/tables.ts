//! The replica's rows, one map per entity, and the protocol's rules for changing them.

import {
	ENTITIES,
	type Entity,
	type Key,
	type Mutation,
	type Rows,
} from "./protocol";

export type Tables = { [E in Entity]: Map<Key, Rows[E]> };

export function emptyTables(): Tables {
	return {
		workspace: new Map(),
		folder: new Map(),
		bookmark: new Map(),
		bookmark_tag: new Map(),
		bookmark_folder: new Map(),
		bookmark_text: new Map(),
	};
}

/** A shallow copy: the maps are new, the rows are shared (rows are never mutated). */
export function copyTables(tables: Tables): Tables {
	const copy = emptyTables();
	for (const entity of ENTITIES) {
		(copy[entity] as Map<Key, unknown>) = new Map<Key, unknown>(
			tables[entity] as Map<Key, unknown>,
		);
	}
	return copy;
}

/** Columns a row can't be made without. Everything else falls back to a default. */
const REQUIRED: { [E in Entity]: (keyof Rows[E])[] } = {
	workspace: ["id", "name"],
	folder: ["id", "workspace_id", "name"],
	bookmark: ["id", "workspace_id", "url"],
	bookmark_tag: ["bookmark_id", "tag"],
	bookmark_folder: ["bookmark_id", "folder_id"],
	bookmark_text: ["bookmark_id", "text"],
};

const DEFAULTS: { [E in Entity]: Partial<Rows[E]> } = {
	workspace: {
		color: 0,
		position: 0,
		instructions: "",
		screening: false,
		guideline: "",
		only_listed_tags: false,
		tag_list: [],
	},
	folder: {
		description: "",
		auto_assign: true,
		color: 0,
		hue: null,
		icon: null,
		position: 0,
	},
	bookmark: {
		title: "",
		site: "",
		added_at: 0,
		up_next: false,
		kept: false,
		deleted_at: null,
		browser_folder: null,
		status: "new",
		kind: null,
		summary: null,
		fields: {},
		extras: [],
		set_aside: null,
		model: null,
		organized_at: null,
		error: null,
		image_key: null,
		image_width: null,
		image_height: null,
	},
	bookmark_tag: { origin: "user", excluded: false },
	bookmark_folder: { origin: "user", probability: null, excluded: false },
	bookmark_text: {},
};

/**
 * Applies a `put`: the columns in `data` over the current row. A new row needs its required
 * columns, so a stray update to a row this replica doesn't have changes nothing.
 */
export function putRow<E extends Entity>(
	tables: Tables,
	entity: E,
	key: Key,
	data: Partial<Rows[E]>,
): boolean {
	const map = tables[entity] as Map<Key, Rows[E]>;
	const current = map.get(key);
	if (current) {
		map.set(key, { ...current, ...data });
		return true;
	}
	const complete = REQUIRED[entity].every((column) => data[column] != null);
	if (!complete) return false;
	map.set(key, { ...DEFAULTS[entity], ...data } as Rows[E]);
	return true;
}

/**
 * Deletes a row and, as the protocol says, what hangs off it: a bookmark's tags, folder links
 * and text; a folder's links; a workspace's folders and bookmarks. Returns every key removed.
 */
export function deleteRow(
	tables: Tables,
	entity: Entity,
	key: Key,
): { entity: Entity; key: Key }[] {
	const removed: { entity: Entity; key: Key }[] = [];
	const remove = (e: Entity, k: Key) => {
		if (tables[e].delete(k)) removed.push({ entity: e, key: k });
	};

	const workspaces = new Set<string>();
	const folders = new Set<string>();
	const bookmarks = new Set<string>();
	if (entity === "workspace") workspaces.add(key);
	else if (entity === "folder") folders.add(key);
	else if (entity === "bookmark") bookmarks.add(key);
	// Children still cascade when the parent itself isn't here: they may have arrived first.
	remove(entity, key);

	if (workspaces.size > 0) {
		for (const [k, folder] of tables.folder) {
			if (workspaces.has(folder.workspace_id)) folders.add(k);
		}
		for (const [k, bookmark] of tables.bookmark) {
			if (workspaces.has(bookmark.workspace_id)) bookmarks.add(k);
		}
	}
	for (const k of folders) remove("folder", k);
	for (const k of bookmarks) {
		remove("bookmark", k);
		remove("bookmark_text", k);
	}
	if (bookmarks.size > 0) {
		for (const [k, link] of tables.bookmark_tag) {
			if (bookmarks.has(link.bookmark_id)) remove("bookmark_tag", k);
		}
	}
	if (bookmarks.size > 0 || folders.size > 0) {
		for (const [k, link] of tables.bookmark_folder) {
			if (bookmarks.has(link.bookmark_id) || folders.has(link.folder_id)) {
				remove("bookmark_folder", k);
			}
		}
	}
	return removed;
}

/** Applies one mutation as the server would for a user. */
export function applyMutation(tables: Tables, mutation: Mutation): void {
	if (mutation.op === "delete") {
		deleteRow(tables, mutation.entity, mutation.key);
	} else {
		putRow(tables, mutation.entity, mutation.key, mutation.data);
	}
}
