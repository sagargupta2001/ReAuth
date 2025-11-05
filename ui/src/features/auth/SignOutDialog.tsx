import { useLocation, useNavigate } from 'react-router-dom'

import { ConfirmDialog } from '@/components/confirm-dialog'
import { sessionStore } from '@/entities/session/model/sessionStore'

interface SignOutDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SignOutDialog({ open, onOpenChange }: SignOutDialogProps) {
  const navigate = useNavigate()
  const location = useLocation()
  const { auth } = sessionStore()

  const handleSignOut = () => {
    auth.reset()

    // Preserve current location for redirect after sign-in
    const currentPath = location.pathname + location.search
    navigate(`/sign-in?redirect=${encodeURIComponent(currentPath)}`, { replace: true })
  }

  return (
    <ConfirmDialog
      open={open}
      onOpenChange={onOpenChange}
      title="Sign out"
      desc="Are you sure you want to sign out? You will need to sign in again to access your account."
      confirmText="Sign out"
      destructive
      handleConfirm={handleSignOut}
      className="sm:max-w-sm"
    />
  )
}
