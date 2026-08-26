/** One selectable option on a radio group or a select. */
export interface FluidChoice {
  value: string
  label: string
}

/**
 * Parses a block's authored options list.
 *
 * The format is one option per line, `value|label`, with the label defaulting
 * to the value. A textarea is a deliberate stop-gap: a proper repeater control
 * is the right editor for this and is worth building once a third block needs
 * options, but a stringly list should not block shipping radio and select.
 *
 * Blank lines are skipped rather than becoming empty options, because a
 * trailing newline is what a textarea gives you for free.
 */
export function parseChoices(raw: unknown): FluidChoice[] {
  if (Array.isArray(raw)) {
    // A hand-authored blueprint may already carry structured options.
    return raw
      .map((entry) => {
        if (typeof entry === 'string') return { value: entry, label: entry }
        const record = (entry ?? {}) as Record<string, unknown>
        const value = String(record.value ?? '')
        return { value, label: String(record.label ?? value) }
      })
      .filter((choice) => choice.value !== '')
  }

  return String(raw ?? '')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const separator = line.indexOf('|')
      if (separator === -1) return { value: line, label: line }
      const value = line.slice(0, separator).trim()
      const label = line.slice(separator + 1).trim()
      return { value, label: label || value }
    })
    .filter((choice) => choice.value !== '')
}
