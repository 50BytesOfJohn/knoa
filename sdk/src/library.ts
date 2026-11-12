//! The read model the UI renders: the replica's rows with orphans left out and links grouped.

import type {
	Bookmark,
	BookmarkFolder,
	BookmarkId,
	BookmarkTag,
	Folder,
	FolderId,
	Workspace,
	WorkspaceId,
} from "./protocol";
import type { Tables } from "./tables";

export interface Library {
	/** In switcher order. */
	workspaces: Workspace[];
	workspace: Map<WorkspaceId, Workspace>;
	folder: Map<FolderId, Folder>;
	bookmark: Map<BookmarkId, Bookmark>;
	/** Every tag row of a bookmark, removed ones (`excluded`) included. */
	tags: Map<BookmarkId, BookmarkTag[]>;
	/** Every folder link of a bookmark, removed ones (`excluded`) included. */
	folders: Map<BookmarkId, BookmarkFolder[]>;
}

/**
 * Rows can arrive before their parents, so a row only shows once everything above it is
 * here. Until then it waits in the replica.
 */
export function buildLibrary(tables: Tables): Library {
	const workspace = new Map(tables.workspace);
	const folder = new Map<FolderId, Folder>();
	for (const [id, row] of tables.folder) {
		if (workspace.has(row.workspace_id)) folder.set(id, row);
	}
	const bookmark = new Map<BookmarkId, Bookmark>();
	for (const [id, row] of tables.bookmark) {
		if (workspace.has(row.workspace_id)) bookmark.set(id, row);
	}
	const tags = new Map<BookmarkId, BookmarkTag[]>();
	for (const row of tables.bookmark_tag.values()) {
		if (bookmark.has(row.bookmark_id)) push(tags, row.bookmark_id, row);
	}
	const folders = new Map<BookmarkId, BookmarkFolder[]>();
	for (const row of tables.bookmark_folder.values()) {
		if (bookmark.has(row.bookmark_id) && folder.has(row.folder_id)) {
			push(folders, row.bookmark_id, row);
		}
	}
	const workspaces = [...workspace.values()].sort(
		(a, b) => a.position - b.position || a.name.localeCompare(b.name),
	);
	return { workspaces, workspace, folder, bookmark, tags, folders };
}

function push<K, V>(map: Map<K, V[]>, key: K, value: V) {
	const list = map.get(key);
	if (list) list.push(value);
	else map.set(key, [value]);
}
