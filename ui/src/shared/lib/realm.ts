const NON_REALM_ROUTES = new Set(['login', 'register', 'forgot-password', 'setup'])

function normalizePath(path: string): string {
  const trimmed = path.trim()
  if (!trimmed) return ''
  const noHash = trimmed.startsWith('#') ? trimmed.slice(1) : trimmed
  const [clean] = noHash.split('?')
  return clean
}

function extractRealmFromPath(path: string): string | null {
  const clean = normalizePath(path)
  if (!clean) return null
  const segment = clean.replace(/^\/+/, '').split('/').filter(Boolean)[0]
  if (!segment || NON_REALM_ROUTES.has(segment)) return null
  return segment
}

function extractRealmFromRedirect(redirect: string | null): string | null {
  if (!redirect) return null
  let value = redirect
  try {
    value = decodeURIComponent(redirect)
  } catch {
    // ignore decode errors
  }
  return extractRealmFromPath(value)
}

export function resolveRealmFromLocation(fallback: string): string {
  try {
    const searchParams = new URLSearchParams(window.location.search)
    const directRealm = searchParams.get('realm')
    if (directRealm) return directRealm

    const redirectRealm = extractRealmFromRedirect(searchParams.get('redirect'))
    if (redirectRealm) return redirectRealm

    const hash = window.location.hash || ''
    const hashPath = hash.startsWith('#') ? hash.slice(1) : hash

    if (hashPath.includes('?')) {
      const queryPart = hashPath.split('?')[1]
      const hashParams = new URLSearchParams(queryPart)
      const hashRealm = hashParams.get('realm')
      if (hashRealm) return hashRealm
      const hashRedirectRealm = extractRealmFromRedirect(hashParams.get('redirect'))
      if (hashRedirectRealm) return hashRedirectRealm
    }

    const pathRealm = extractRealmFromPath(hashPath)
    if (pathRealm) return pathRealm
  } catch {
    // ignore parsing errors
  }

  return fallback
}

export function isNonRealmRoute(segment: string): boolean {
  return NON_REALM_ROUTES.has(segment)
}
