import { useNavigate, type NavigateOptions } from 'react-router-dom'
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
