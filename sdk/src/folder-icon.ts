//! What a folder's `icon` column means: one of Knoa's glyphs, one emoji, or nothing.

import { MAX_CHARS } from "./protocol";

/** The glyph names Knoa draws, in picker order, as `FolderGlyph` in knoa-core. */
export const FOLDER_GLYPHS = [
	"read",
	"article",
	"code",
	"write",
	"work",
	"tag",
	"bag",
	"plane",
	"cup",
	"music",
	"play",
	"photo",
	"bulb",
	"heart",
	"home",
	"star",
	"clock",
	"flask",
	"globe",
	"person",
	"box",
	"inbox",
	"leaf",
	"bolt",
	"lock",
	"pin",
	"chat",
	"game",
] as const;

export type FolderGlyph = (typeof FOLDER_GLYPHS)[number];

/** A glyph is tinted with the folder's color; an emoji keeps its own. */
export type FolderIcon =
	| { type: "glyph"; name: FolderGlyph }
	| { type: "emoji"; emoji: string };

const glyphs: ReadonlySet<string> = new Set(FOLDER_GLYPHS);

/** `null` for anything that is neither a known glyph nor one emoji: the folder shape. */
export function parseFolderIcon(value: string | null | undefined): FolderIcon | null {
	if (!value) return null;
	if (glyphs.has(value)) return { type: "glyph", name: value as FolderGlyph };
	return isEmoji(value) ? { type: "emoji", emoji: value } : null;
}

const graphemes = new Intl.Segmenter(undefined, { granularity: "grapheme" });
const EMOJI = /\p{Extended_Pictographic}|\p{Regional_Indicator}|⃣/u;

/** Exactly one emoji: a single grapheme cluster that is pictographic, a flag or a keycap. */
export function isEmoji(text: string): boolean {
	if ([...text].length > MAX_CHARS.folder.icon) return false;
	const segments = [...graphemes.segment(text)];
	return segments.length === 1 && EMOJI.test(text);
}
