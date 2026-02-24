export function fuzzyScore(query: string, text: string): number {
  const q = query.trim().toLowerCase()
  const t = text.trim().toLowerCase()

  if (!q || !t) return 0

  let score = 0
  let tIndex = 0
  let lastMatch = -1
  let streak = 0

  for (let i = 0; i < q.length; i += 1) {
    const ch = q[i]
    const idx = t.indexOf(ch, tIndex)
    if (idx === -1) return 0

    if (idx === lastMatch + 1) {
      streak += 1
      score += 2 + streak
    } else {
      streak = 0
      score += 1
    }

    if (idx === 0) {
      score += 2
    }

    lastMatch = idx
    tIndex = idx + 1
  }

  if (t.startsWith(q)) {
    score += q.length
  }

  return score
}

export function buildHaystack(parts: Array<string | undefined | null>) {
  return parts.filter(Boolean).join(' ').trim()
}

export function rankItems<T>(
  items: T[],
  query: string,
  getText: (item: T) => string,
  getId: (item: T) => string,
  recencyMap: Map<string, number>,
) {
  const q = query.trim()
  const scored = items
    .map((item, index) => {
      const text = getText(item)
      const score = q ? fuzzyScore(q, text) : 0
      const recency = recencyMap.get(getId(item)) || 0
      return {
        item,
        score: score + recency,
        matches: score > 0,
        index,
      }
    })
    .filter((entry) => (q ? entry.matches : true))

  return scored
    .sort((a, b) => (b.score - a.score) || a.index - b.index)
    .map((entry) => entry.item)
}
