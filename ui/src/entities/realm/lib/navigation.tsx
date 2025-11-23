import { forwardRef } from 'react'

import { Link, type LinkProps, type NavigateOptions, useNavigate } from 'react-router-dom'

import { useActiveRealm } from '../model/useActiveRealm'

/**
 * Helper to format the path with the realm.
 * Input: "/create-realm" -> Output: "/master/create-realm"
 */
export function getRealmPath(path: string, realm: string) {
  // Don't touch absolute URLs
  if (path.startsWith('http')) return path

  // Handle root path
  const cleanPath = path === '/' ? '' : path

  // Ensure path starts with / if not empty
  const normalizedPath = cleanPath.startsWith('/') || cleanPath === '' ? cleanPath : `/${cleanPath}`

  return `/${realm}${normalizedPath}`
}

/**
 * A wrapper around React Router's Link that automatically
 * prepends the current realm to the `to` prop.
 */
export const RealmLink = forwardRef<HTMLAnchorElement, LinkProps>(({ to, ...props }, ref) => {
  const realm = useActiveRealm()

  // Only modify string paths. Object paths are left as-is for advanced use cases.
  const scopedTo = typeof to === 'string' ? getRealmPath(to, realm) : to

  return <Link ref={ref} to={scopedTo} {...props} />
})
RealmLink.displayName = 'RealmLink'

/**
 * A wrapper around useNavigate that automatically scopes string paths.
 */
export function useRealmNavigate() {
  const navigate = useNavigate()
  const realm = useActiveRealm()

  return (to: string | number, options?: NavigateOptions) => {
    if (typeof to === 'number') {
      navigate(to as number) // Handle "go back" logic (-1)
      return
    }

    navigate(getRealmPath(to, realm), options)
  }
}
