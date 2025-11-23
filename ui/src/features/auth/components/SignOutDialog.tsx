import { useLocation, useNavigate } from 'react-router-dom'

import { ConfirmDialog } from '@/shared/ui/confirm-dialog.tsx'

interface SignOutDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SignOutDialog({ open, onOpenChange }: SignOutDialogProps) {
  const navigate = useNavigate()
  const location = useLocation()

  const handleSignOut = () => {
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
