/**
 * Client list fields (redirect_uris, web_origins, scopes) arrive as JSON array
 * strings from the API (e.g. '["openid","profile"]'). This safely parses them,
 * tolerating a legacy space-separated form and malformed values.
 */
export function parseJsonArray(raw: string | null | undefined): string[] {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    if (Array.isArray(parsed)) {
      return parsed.filter((v): v is string => typeof v === 'string' && v.length > 0)
    }
  } catch {
    // Fall back to whitespace-separated tokens.
    return raw.split(/\s+/).filter(Boolean)
  }
  return []
}
