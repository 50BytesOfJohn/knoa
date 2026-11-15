# 0016: Images are stored and shown to the AI as derived copies

- Status: Accepted
- Date: 2026-11-15
- Amends: [0009](0009-provider-neutral-ai.md) (messages are text only)

## Context

A bookmark that links straight to an image file was organized from its URL alone, so its
title stayed the URL and its tags and summary were guesses. Its original was cached, shown
and uploaded: GPUI decoded 8K wallpapers at full size, and sync moved files of many
megabytes. The same fetch code runs in the cloud organizer, a 128 MB Cloudflare Worker.

Measured on OpenRouter, a 1024 pixel JPEG at "high" detail cost about as much as a text
page with the default models; the original URL cost up to ten times the prompt tokens.

## Decision

- `knoa-fetch` downloads a linked image once and decodes it once, within a pixel and
  memory budget checked from its headers first, with JPEGs scaled down in the DCT. From
  that decode come a display copy (at most 1600 pixels, always a still) and an AI copy
  (1024 pixel JPEG). The display copy is what is cached, shown and uploaded. Files that
  can't be decoded are described from their header only.
- `knoa-ai` messages hold text or parts (text and images), in a form no provider owns.
  OpenRouter gets images inline as data URLs at "high" detail. The classifier stays text
  only; the summary and tags are enough for folders.
- An image is one organize run with the same models. When no configured model takes
  images, the run is repeated once without the image.

## Consequences

- Image bookmarks get real tags, summaries, fields and titles at about the cost of a
  page, and never send an original to a provider.
- The original isn't kept: the display copy is enough for cards and detail, and the
  original stays at its URL.
- The `image` crate and `jpeg-decoder` add about 200 KB, compressed, to the Worker.
- A model a user picks that can't see images costs one rejected request per image.
