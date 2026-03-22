import { useEffect, useRef, useState } from 'react'

import { useLocation, useNavigate } from 'react-router-dom'

import { Button } from '@/components/button'
import {
  SETUP_COMPLETE_EVENT,
  clearSetupStatusCache,
  getSetupRequired,
  markSetupSealed,
} from '@/shared/lib/setupStatus'

type SetupGateProps = {
  children: React.ReactNode
}

type GateState = 'checking' | 'required' | 'sealed' | 'error'

export function SetupGate({ children }: SetupGateProps) {
  const location = useLocation()
  const navigate = useNavigate()
  const [state, setState] = useState<GateState>('checking')
  const [error, setError] = useState<string | null>(null)
  const redirectingRef = useRef(false)

  useEffect(() => {
    let active = true

    const run = async () => {
      try {
        const required = await getSetupRequired()
        if (!active) return
        setState(required ? 'required' : 'sealed')
      } catch (err) {
        if (!active) return
        setError(err instanceof Error ? err.message : 'Failed to check setup status.')
        setState('error')
      }
    }

    void run()

    return () => {
      active = false
    }
  }, [])

  useEffect(() => {
    const onComplete = () => {
      markSetupSealed()
      setState('sealed')
    }
    window.addEventListener(SETUP_COMPLETE_EVENT, onComplete)
    return () => {
      window.removeEventListener(SETUP_COMPLETE_EVENT, onComplete)
    }
  }, [])

  useEffect(() => {
    if (redirectingRef.current) return

    const isSetupRoute = location.pathname === '/setup' || location.pathname === '/setup/'

    if (state === 'required' && !isSetupRoute) {
      redirectingRef.current = true
      navigate('/setup', { replace: true })
    }

    if (state === 'sealed' && isSetupRoute) {
      redirectingRef.current = true
      navigate('/login', { replace: true })
    }
  }, [location.pathname, navigate, state])

  if (state === 'checking') {
    return <div className="flex h-screen items-center justify-center">Checking setup...</div>
  }

  if (state === 'error') {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-3 text-sm text-muted-foreground">
        <p>{error || 'Failed to check setup status.'}</p>
        <Button
          variant="outline"
          onClick={() => {
            redirectingRef.current = false
            setError(null)
            setState('checking')
            clearSetupStatusCache()
            void getSetupRequired()
              .then((required) => {
                setState(required ? 'required' : 'sealed')
              })
              .catch((err) => {
                setError(err instanceof Error ? err.message : 'Failed to check setup status.')
                setState('error')
              })
          }}
        >
          Retry
        </Button>
      </div>
    )
  }

  return <>{children}</>
}
