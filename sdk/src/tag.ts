// A port of `knoa_core::normalize_tag` (crates/knoa-core/src/tag.rs in the knoa repo). Both
// cores must spell a tag the same way, or the same tag syncs as two rows.

/** Rust's `char::is_whitespace` (Unicode White_Space), which differs from JS `\s`. */
const SPACE =
	"\\t\\n\\v\\f\\r \\u0085\\u00a0\\u1680\\u2000-\\u200a\\u2028\\u2029\\u202f\\u205f\\u3000";
const EDGE_SPACE = new RegExp(`^[${SPACE}]+|[${SPACE}]+$`, "gu");
const SEPARATOR = new RegExp(`[${SPACE}_-]`, "u");

/** Lowercase, trimmed, words joined by hyphens: `"Film Noir "` → `"film-noir"`. */
export function normalizeTag(tag: string): string {
	return tag
		.replace(EDGE_SPACE, "")
		.replace(/^#+/u, "")
		.split(SEPARATOR)
		.filter((part) => part !== "")
		.join("-")
		.toLowerCase();
}

/** Anything longer is a phrase, not a tag. */
export const MAX_TAG_CHARS = 40;
