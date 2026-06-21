import { useEffect, useMemo } from 'react'

import { Loader2, Workflow } from 'lucide-react'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useFlows } from '@/features/flow/api/useFlows.ts'
import { useFlowBindings } from '@/features/flow/hooks/useFlowBindings.ts'

export function FlowsIndexPage() {
  const navigate = useRealmNavigate()
  const { isFlowActive } = useFlowBindings()
  const { data: flows, isLoading } = useFlows()

  // Preselect a flow so the page never lands on an empty shell. Prefer an
  // active/bound flow, falling back to the first flow alphabetically.
  const defaultFlowId = useMemo(() => {
    if (!flows?.length) return undefined
    const sorted = [...flows].sort((a, b) => a.alias.localeCompare(b.alias))
    return (sorted.find((f) => isFlowActive(f)) ?? sorted[0]).id
  }, [flows, isFlowActive])

  useEffect(() => {
    if (defaultFlowId) navigate(`/flows/${defaultFlowId}/overview`, { replace: true })
  }, [defaultFlowId, navigate])

  if (isLoading || defaultFlowId) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Flow...</p>
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col items-center justify-center space-y-4 text-center">
      <div className="bg-muted flex h-20 w-20 items-center justify-center rounded-full">
        <Workflow className="text-muted-foreground h-10 w-10" />
      </div>
      <div className="max-w-md space-y-2">
        <h2 className="text-2xl font-bold tracking-tight">Authentication Flows</h2>
        <p className="text-muted-foreground">
          Flows define the sequence of actions a user must take to authenticate. Create your first
          flow from the sidebar to get started.
        </p>
      </div>
    </div>
  )
}
