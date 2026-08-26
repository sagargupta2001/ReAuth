/** A run of legal copy: either plain text or a link. */
export type InlineSegment =
  | { kind: 'text'; text: string }
  | { kind: 'link'; text: string; href: string }

const LINK_PATTERN = /\[([^\]]+)\]\(([^)\s]+)\)/g

/**
 * Splits legal copy into text and links.
 *
 * Consent lines are the case a plain `Text` block cannot express: "I accept the
 * [Terms](/terms) and [Privacy Policy](/privacy)" would otherwise need a row of
 * four alternating Text and Link blocks, which is miserable to author and
 * breaks as soon as the wording changes.
 *
 * Only `[label](href)` is recognised. Everything else — including a malformed
 * bracket — stays literal, because silently swallowing text a builder typed is
 * worse than showing it.
 */
export function parseInlineLinks(raw: unknown): InlineSegment[] {
  const text = String(raw ?? '')
  if (!text) return []

  const segments: InlineSegment[] = []
  let cursor = 0

  for (const match of text.matchAll(LINK_PATTERN)) {
    const start = match.index ?? 0
    if (start > cursor) {
      segments.push({ kind: 'text', text: text.slice(cursor, start) })
    }
    segments.push({ kind: 'link', text: match[1], href: match[2] })
    cursor = start + match[0].length
  }

  if (cursor < text.length) {
    segments.push({ kind: 'text', text: text.slice(cursor) })
  }
  return segments
}
