import { formatDistanceToNow } from 'date-fns'
import { Clock, Copy, Loader2, RotateCcw, ShieldCheck } from 'lucide-react'

import { Button } from '@/components/button'
import { Skeleton } from '@/components/skeleton'
import { useFlowVersions } from '@/features/flow-builder/api/useFlowVersions'
import { useRollbackFlow } from '@/features/flow-builder/api/useRollbackFlow.ts'
import { useRestoreDraft } from '@/features/flow/api/useRestoreDraft.ts'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/shared/ui/alert-dialog.tsx'

interface FlowHistoryTabProps {
  flowId: string
  activeVersion?: number | null
}

export function FlowHistoryTab({ flowId, activeVersion }: FlowHistoryTabProps) {
  const { data, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage } =
    useFlowVersions(flowId)

  const { mutate: restoreDraft, isPending: isRestoring } = useRestoreDraft()

  const { mutate: rollback, isPending } = useRollbackFlow()

  const versions = data?.pages || []

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
        <div className="bg-muted/30 sticky top-0 z-10 border-b p-4">
          <h3 className="font-semibold">Deployment History</h3>
          <p className="text-muted-foreground text-sm">
            History of all published versions of this flow.
          </p>
        </div>

        <div className="max-h-[calc(100vh-340px)] divide-y overflow-y-auto">
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
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <Button variant="outline" size="sm" className="h-8 text-xs">
                          <RotateCcw className="mr-2 h-3 w-3" />
                          Rollback
                        </Button>
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>
                            Rollback to Version {version.version_number}?
                          </AlertDialogTitle>
                          <AlertDialogDescription>
                            This will immediately change the active login flow for all users to this
                            version. Your current draft will NOT be affected.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction onClick={() => rollback(version.version_number)}>
                            {isPending ? 'Rolling back...' : 'Confirm Rollback'}
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  )}

                  {/* RESTORE DRAFT BUTTON (Design Action) */}
                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button
                        size="sm"
                        className="h-7 text-xs"
                        title="Edit this version"
                        variant="soft"
                      >
                        <Copy className="mr-2 h-3 w-3" />
                        Start Draft From Here
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>
                          Revert Draft to Version {version.version_number}?
                        </AlertDialogTitle>
                        <AlertDialogDescription>
                          This will <strong>overwrite your current work-in-progress draft</strong>{' '}
                          with the graph from Version {version.version_number}.
                          <br />
                          <br />
                          The active login flow will NOT change. This only affects the builder.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction onClick={() => restoreDraft(version.version_number)}>
                          {isRestoring ? 'Restoring...' : 'Overwrite Draft'}
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </div>
              </div>
            )
          })}
        </div>

        {hasNextPage && (
          <div className="flex justify-center border-t p-4">
            <Button
              variant="outline"
              size="sm"
              onClick={() => fetchNextPage()}
              disabled={isFetchingNextPage}
            >
              {isFetchingNextPage ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Loading...
                </>
              ) : (
                'Load Older Versions'
              )}
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}
