import { useEffect, useRef } from 'react'
import { useQueryClient } from '@tanstack/react-query'

import { useLocation, useNavigate } from 'react-router-dom'

import { Button } from '@/components/button'
import { useSetupStatus } from '@/features/setup/api/useSetupStatus'
import { SETUP_COMPLETE_EVENT, clearSetupSeal, markSetupSealed } from '@/shared/lib/setupStatus'
import { queryKeys } from '@/shared/lib/queryKeys'

type SetupGateProps = {
  children: React.ReactNode
}

export function SetupGate({ children }: SetupGateProps) {
  const location = useLocation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const setupStatus = useSetupStatus()
  const redirectingRef = useRef(false)

  useEffect(() => {
    const onComplete = () => {
      markSetupSealed()
      queryClient.setQueryData(queryKeys.setupStatus(), { required: false })
    }
    window.addEventListener(SETUP_COMPLETE_EVENT, onComplete)
    return () => {
      window.removeEventListener(SETUP_COMPLETE_EVENT, onComplete)
    }
  }, [queryClient])

  useEffect(() => {
    if (redirectingRef.current) return

    const isSetupRoute = location.pathname === '/setup' || location.pathname === '/setup/'
    const state = setupStatus.isLoading
      ? 'checking'
      : setupStatus.isError
        ? 'error'
        : setupStatus.data?.required
          ? 'required'
          : 'sealed'

    if (state === 'required' && !isSetupRoute) {
      redirectingRef.current = true
      navigate('/setup', { replace: true })
    }

    if (state === 'sealed' && isSetupRoute) {
      redirectingRef.current = true
      navigate('/login', { replace: true })
    }
  }, [location.pathname, navigate, setupStatus.data?.required, setupStatus.isError, setupStatus.isLoading])

  if (setupStatus.isLoading) {
    return <div className="flex h-screen items-center justify-center">Checking setup...</div>
  }

  if (setupStatus.isError) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-3 text-sm text-muted-foreground">
        <p>{setupStatus.error?.message || 'Failed to check setup status.'}</p>
        <Button
          variant="outline"
          onClick={() => {
            redirectingRef.current = false
            clearSetupSeal()
            void setupStatus.refetch()
          }}
        >
          Retry
        </Button>
      </div>
    )
  }

  return <>{children}</>
}
