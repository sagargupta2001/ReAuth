import { useEffect } from 'react'

const HASH_ROUTES = new Set(['/login', '/register', '/forgot-password', '/invite/accept', '/setup'])

export function HashRouteNormalizer() {
  useEffect(() => {
    const { pathname, search, hash, origin } = window.location
    if (!HASH_ROUTES.has(pathname)) {
      return
    }

    const hashPath = hash.startsWith('#') ? hash.slice(1) : hash
    if (hashPath === pathname) {
      return
    }

    window.location.replace(`${origin}/#${pathname}${search}`)
  }, [])

  return null
}
