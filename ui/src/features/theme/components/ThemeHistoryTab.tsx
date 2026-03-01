import { formatDistanceToNow } from 'date-fns'
import { Clock, Loader2, ShieldCheck } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Skeleton } from '@/components/skeleton'
import { useActivateThemeVersion } from '@/features/theme/api/useActivateThemeVersion'
import { useStartThemeDraftFromVersion } from '@/features/theme/api/useStartThemeDraftFromVersion'
import { useThemeVersions } from '@/features/theme/api/useThemeVersions'

interface ThemeHistoryTabProps {
  themeId: string
  activeVersionId?: string | null
}

export function ThemeHistoryTab({ themeId, activeVersionId }: ThemeHistoryTabProps) {
  const { data: versions, isLoading, isFetching } = useThemeVersions(themeId)
  const { mutate: activateVersion, isPending } = useActivateThemeVersion(themeId)
  const { mutate: startDraft, isPending: isStartingDraft } = useStartThemeDraftFromVersion(themeId)

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
        <p className="text-xs">Publish your first theme to create a version.</p>
      </div>
    )
  }

  return (
    <div className="p-6">
      <div className="bg-card rounded-md border">
        <div className="bg-muted/30 sticky top-0 z-10 border-b p-4">
          <h3 className="font-semibold">Deployment History</h3>
          <p className="text-muted-foreground text-sm">
            History of all published versions of this theme.
          </p>
        </div>

        <div className="max-h-[calc(100vh-340px)] divide-y overflow-y-auto">
          {versions.map((version) => {
            const isActive = version.id === activeVersionId

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
                      {isActive && (
                        <Badge variant="secondary" className="h-4 px-2 text-[9px]">
                          Active
                        </Badge>
                      )}
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
                  <Button
                    variant={isActive ? 'secondary' : 'outline'}
                    size="sm"
                    className="h-8 text-xs"
                    disabled={isActive || isPending}
                    onClick={() => activateVersion(version.id)}
                  >
                    {isActive ? 'Active' : 'Rollback'}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 text-xs"
                    disabled={isPending || isStartingDraft}
                    onClick={() => startDraft(version.id)}
                  >
                    Start draft from here
                  </Button>
                </div>
              </div>
            )
          })}
        </div>

        {isFetching && (
          <div className="flex justify-center border-t p-4">
            <div className="text-muted-foreground flex items-center gap-2 text-xs">
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
              Refreshing history...
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
