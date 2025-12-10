import { formatDistanceToNow } from 'date-fns'
import { Clock, RotateCcw, ShieldCheck } from 'lucide-react'

import { Button } from '@/components/button'
import { Skeleton } from '@/components/skeleton'
import { useFlowVersions } from '@/features/flow-builder/api/useFlowVersions'

// Assuming you have shadcn skeleton

interface FlowHistoryTabProps {
  flowId: string
  activeVersion?: number | null
}

export function FlowHistoryTab({ flowId, activeVersion }: FlowHistoryTabProps) {
  const { data: versions, isLoading } = useFlowVersions(flowId)

  if (isLoading) {
    return (
      <div className="space-y-2 p-6">
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-12 w-full" />
      </div>
    )
  }

  if (!versions || versions.length === 0) {
    return (
      <div className="text-muted-foreground flex flex-col items-center justify-center gap-2 p-12 text-center">
        <ShieldCheck className="h-10 w-10 opacity-20" />
        <p>No version history available.</p>
        <p className="text-xs">Publish your first draft to create a version.</p>
      </div>
    )
  }

  return (
    <div className="p-6">
      <div className="bg-card rounded-md border">
        <div className="bg-muted/30 border-b p-4">
          <h3 className="font-semibold">Deployment History</h3>
          <p className="text-muted-foreground text-sm">
            History of all published versions of this flow.
          </p>
        </div>

        <div className="divide-y">
          {versions.map((version) => {
            const isActive = version.version_number === activeVersion

            return (
              <div
                key={version.id}
                className="hover:bg-muted/5 flex items-center justify-between p-4 transition-colors"
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`flex h-8 w-8 items-center justify-center rounded-full border ${isActive ? 'border-green-200 bg-green-100' : 'bg-muted border-transparent'}`}
                  >
                    <span
                      className={`text-xs font-bold ${isActive ? 'text-green-700' : 'text-muted-foreground'}`}
                    >
                      v{version.version_number}
                    </span>
                  </div>

                  <div className="flex flex-col gap-0.5">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium">
                        Published Version {version.version_number}
                      </span>
                    </div>
                    <div className="text-muted-foreground flex items-center gap-1 text-xs">
                      <Clock className="h-3 w-3" />
                      <span>
                        {formatDistanceToNow(new Date(version.created_at), { addSuffix: true })}
                      </span>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  {!isActive && (
                    <Button variant="outline" size="sm" className="h-8 text-xs" disabled>
                      <RotateCcw className="mr-2 h-3 w-3" />
                      Rollback
                    </Button>
                  )}
                  {/* You could add a "View Snapshot" button here later to open read-only builder */}
                </div>
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
