import { forwardRef } from 'react'

import { Link, type LinkProps } from 'react-router-dom'

import { useActiveRealm } from '../model/useActiveRealm'
import { getRealmPath } from './navigation.logic'

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
