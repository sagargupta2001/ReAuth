import { formatDistanceToNow } from 'date-fns'
import { Clock, Loader2, ShieldCheck } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'

import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'
import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { Skeleton } from '@/components/skeleton'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import type { ThemeVersion } from '@/entities/theme/model/types'
import { useActivateThemeVersion } from '@/features/theme/api/useActivateThemeVersion'
import { useStartThemeDraftFromVersion } from '@/features/theme/api/useStartThemeDraftFromVersion'
import { useThemeTemplateGaps } from '@/features/theme/api/useThemeTemplateGaps'
import { useThemeVersionSnapshot } from '@/features/theme/api/useThemeVersionSnapshot'
import { useThemeVersions } from '@/features/theme/api/useThemeVersions'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

interface ThemeHistoryTabProps {
  themeId: string
  activeVersionId?: string | null
}

export function ThemeHistoryTab({ themeId, activeVersionId }: ThemeHistoryTabProps) {
  const { data: versions, isLoading, isFetching } = useThemeVersions(themeId)
  const { mutate: activateVersion, isPending } = useActivateThemeVersion(themeId)
  const { mutate: startDraft, isPending: isStartingDraft } = useStartThemeDraftFromVersion(themeId)
  const { data: templateGaps } = useThemeTemplateGaps(themeId)
  const missingTemplates = templateGaps?.missing ?? []
  const [confirmVersion, setConfirmVersion] = useState<ThemeVersion | null>(null)
  const confirmOpen = confirmVersion !== null
  const [snapshotVersionId, setSnapshotVersionId] = useState<string | null>(null)
  const navigate = useRealmNavigate()
  const {
    data: snapshotData,
    isLoading: isSnapshotLoading,
  } = useThemeVersionSnapshot(themeId, snapshotVersionId)
  const activeSnapshotId = snapshotVersionId ? activeVersionId ?? null : null
  const { data: activeSnapshot, isLoading: isActiveSnapshotLoading } = useThemeVersionSnapshot(
    themeId,
    activeSnapshotId,
  )
  const activeVersionNumber = useMemo(
    () => versions?.find((version) => version.id === activeVersionId)?.version_number,
    [versions, activeVersionId],
  )

  type DiffEntry = { path: string; left: unknown; right: unknown }

  const diffEntries = useMemo<DiffEntry[]>(() => {
    const source = activeSnapshot?.snapshot
    const target = snapshotData?.snapshot
    if (!source || !target) return []

    const diffs: DiffEntry[] = []
    const maxDiffs = 200
    const maxDepth = 5

    const record = (path: string, left: unknown, right: unknown) => {
      if (diffs.length >= maxDiffs) return
      diffs.push({ path, left, right })
    }

    const isObject = (value: unknown) =>
      value !== null && typeof value === 'object' && !Array.isArray(value)

    const walk = (left: unknown, right: unknown, path: string, depth: number) => {
      if (diffs.length >= maxDiffs) return
      if (left === right) return
      if (depth >= maxDepth) {
        record(path, left, right)
        return
      }
      if (Array.isArray(left) && Array.isArray(right)) {
        if (left.length !== right.length) {
          record(`${path}.length`, left.length, right.length)
        }
        const maxLen = Math.max(left.length, right.length)
        for (let i = 0; i < maxLen; i += 1) {
          if (i >= left.length) {
            record(`${path}[${i}]`, undefined, right[i])
            continue
          }
          if (i >= right.length) {
            record(`${path}[${i}]`, left[i], undefined)
            continue
          }
          walk(left[i], right[i], `${path}[${i}]`, depth + 1)
          if (diffs.length >= maxDiffs) break
        }
        return
      }
      if (isObject(left) && isObject(right)) {
        const leftObj = left as Record<string, unknown>
        const rightObj = right as Record<string, unknown>
        const keys = new Set([...Object.keys(leftObj), ...Object.keys(rightObj)])
        for (const key of keys) {
          walk(leftObj[key], rightObj[key], path ? `${path}.${key}` : key, depth + 1)
          if (diffs.length >= maxDiffs) break
        }
        return
      }
      record(path, left, right)
    }

    walk(source, target, '', 0)
    return diffs
  }, [activeSnapshot, snapshotData])

  const formatValue = (value: unknown) => {
    if (value === undefined) return 'undefined'
    if (value === null) return 'null'
    if (typeof value === 'string') return `"${value}"`
    try {
      const rendered = JSON.stringify(value)
      return rendered && rendered.length > 120 ? `${rendered.slice(0, 117)}...` : rendered
    } catch {
      return String(value)
    }
  }

  const [diffFilter, setDiffFilter] = useState<'all' | 'tokens' | 'layout' | 'nodes'>('all')
  const filteredDiffs = useMemo(() => {
    if (diffFilter === 'all') return diffEntries
    return diffEntries.filter((entry) => entry.path.startsWith(diffFilter))
  }, [diffEntries, diffFilter])
  const hasDiffs = diffEntries.length > 0

  useEffect(() => {
    if (snapshotVersionId === null) {
      setDiffFilter('all')
    }
  }, [snapshotVersionId])

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
      {missingTemplates.length > 0 && (
        <Alert className="mb-4">
          <AlertTitle>Missing Flow Templates</AlertTitle>
          <AlertDescription>
            The active flows reference templates not defined in this theme. Switching versions will
            fall back to system pages for: {missingTemplates.join(', ')}.
          </AlertDescription>
        </Alert>
      )}
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
                  {!isActive && <Button
                    variant={isActive ? 'secondary' : 'outline'}
                    size="sm"
                    className="h-8 text-xs"
                    disabled={isActive || isPending}
                    onClick={() => {
                      if (missingTemplates.length > 0) {
                        setConfirmVersion(version)
                        return
                      }
                      activateVersion(version.id)
                    }}
                  >
                    Rollback
                  </Button>}
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 text-xs"
                    onClick={() => setSnapshotVersionId(version.id)}
                  >
                    View snapshot
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
      <Dialog
        open={confirmOpen}
        onOpenChange={(open) => {
          if (!open) {
            setConfirmVersion(null)
          }
        }}
      >
      <DialogContent>
          <DialogHeader>
            <DialogTitle>Missing Flow Templates</DialogTitle>
            <DialogDescription>
              The active flows reference templates that are not defined in this theme. Rolling back
              will fall back to system pages for: {missingTemplates.join(', ')}.
            </DialogDescription>
          </DialogHeader>
          {missingTemplates.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {missingTemplates.map((key) => (
                <Button
                  key={key}
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs"
                  onClick={() =>
                    navigate(`/themes/${themeId}/fluid?page=${encodeURIComponent(key)}`)
                  }
                >
                  View “{key}”
                </Button>
              ))}
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmVersion(null)}>
              Cancel
            </Button>
            <Button
              onClick={() => {
                if (confirmVersion) {
                  activateVersion(confirmVersion.id)
                }
                setConfirmVersion(null)
              }}
              disabled={isPending}
            >
              Proceed with rollback
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog
        open={snapshotVersionId !== null}
        onOpenChange={(open) => {
          if (!open) {
            setSnapshotVersionId(null)
          }
        }}
      >
        <DialogContent className="max-w-3xl h-[70vh] min-h-[520px] overflow-hidden">
          <div className="flex h-full min-h-0 flex-col gap-4">
            <DialogHeader>
              <DialogTitle>Theme Snapshot</DialogTitle>
              <DialogDescription>
                Version {snapshotData?.version_number ?? '...'} snapshot payload.
              </DialogDescription>
            </DialogHeader>
            <Tabs
              defaultValue="snapshot"
              className="flex h-full min-h-0 flex-1 flex-col overflow-hidden"
            >
              <TabsList className="mb-3 w-fit self-start">
                <TabsTrigger value="snapshot">Snapshot</TabsTrigger>
                <TabsTrigger value="diff">Diff vs active</TabsTrigger>
              </TabsList>
              <TabsContent
                value="snapshot"
                className="flex h-full min-h-0 flex-1 flex-col overflow-hidden"
              >
                <div className="flex-1 overflow-auto rounded-md border bg-muted/30 p-3 text-xs">
                  {isSnapshotLoading && (
                    <p className="text-muted-foreground">Loading snapshot...</p>
                  )}
                  {!isSnapshotLoading && snapshotData && (
                    <pre className="whitespace-pre-wrap">
                      {JSON.stringify(snapshotData.snapshot, null, 2)}
                    </pre>
                  )}
                </div>
              </TabsContent>
              <TabsContent
                value="diff"
                className="flex h-full min-h-0 flex-1 flex-col overflow-hidden"
              >
                <div className="flex-1 overflow-auto rounded-md border bg-muted/30 p-3 text-xs">
                  {isSnapshotLoading || isActiveSnapshotLoading ? (
                    <p className="text-muted-foreground">Loading diff...</p>
                  ) : !activeSnapshot ? (
                    <p className="text-muted-foreground">
                      Active version snapshot not available.
                    </p>
                  ) : (
                    <div className="space-y-3">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground text-[11px]">
                          Comparing against active version {activeVersionNumber ?? 'unknown'}.
                          Showing {diffFilter === 'all' ? diffEntries.length : filteredDiffs.length} of{' '}
                          {diffEntries.length} change(s).
                        </span>
                        <div className="flex gap-1">
                          {(['all', 'tokens', 'layout', 'nodes'] as const).map((filter) => (
                            <Button
                              key={filter}
                              size="sm"
                              variant={diffFilter === filter ? 'secondary' : 'outline'}
                              className="h-7 text-[11px]"
                              onClick={() => setDiffFilter(filter)}
                            >
                              {filter}
                            </Button>
                          ))}
                        </div>
                      </div>
                      {!hasDiffs ? (
                        <p className="text-muted-foreground">No differences detected.</p>
                      ) : filteredDiffs.length === 0 ? (
                        <p className="text-muted-foreground">
                          No differences for “{diffFilter}”. Try another filter.
                        </p>
                      ) : (
                        <div className="divide-y">
                          {filteredDiffs.map((entry) => {
                            const kind =
                              entry.left === undefined
                                ? 'added'
                                : entry.right === undefined
                                  ? 'removed'
                                  : 'changed'
                            const badgeClass =
                              kind === 'added'
                                ? 'bg-emerald-100 text-emerald-700 border-emerald-200'
                                : kind === 'removed'
                                  ? 'bg-rose-100 text-rose-700 border-rose-200'
                                  : 'bg-amber-100 text-amber-700 border-amber-200'
                            return (
                              <div key={entry.path} className="py-2">
                                <div className="flex items-center gap-2 font-mono text-[11px] text-muted-foreground">
                                  <span>{entry.path || '(root)'}</span>
                                  <span
                                    className={`rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase ${badgeClass}`}
                                  >
                                    {kind}
                                  </span>
                                </div>
                                <div className="grid gap-2 md:grid-cols-2">
                                  <div>
                                    <div className="text-[10px] text-muted-foreground">Active</div>
                                    <div className="rounded-md border bg-background p-2 font-mono text-[11px]">
                                      {formatValue(entry.left)}
                                    </div>
                                  </div>
                                  <div>
                                    <div className="text-[10px] text-muted-foreground">Selected</div>
                                    <div className="rounded-md border bg-background p-2 font-mono text-[11px]">
                                      {formatValue(entry.right)}
                                    </div>
                                  </div>
                                </div>
                              </div>
                            )
                          })}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              </TabsContent>
            </Tabs>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
