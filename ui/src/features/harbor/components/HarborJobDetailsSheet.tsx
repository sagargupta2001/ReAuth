import { useState } from 'react'

import { AlertTriangle, Download, RefreshCcw, ShieldAlert } from 'lucide-react'
import { toast } from 'sonner'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import {
  downloadHarborJobArtifact,
  type HarborJob,
  type HarborJobConflict,
} from '@/features/harbor/api/harborApi'
import { useHarborJobDetails } from '@/features/harbor/api/useHarborJobDetails'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { formatRelativeTime } from '@/lib/utils'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/shared/ui/sheet'

type Props = {
  jobId: string | null
  onOpenChange: (open: boolean) => void
}

function statusBadge(job: HarborJob) {
  switch (job.status.toLowerCase()) {
    case 'completed':
      return { label: 'Completed', variant: 'success' as const }
    case 'failed':
    case 'expired':
      return { label: job.status, variant: 'destructive' as const }
    case 'queued':
    case 'pending':
      return { label: job.status, variant: 'secondary' as const }
    default:
      return { label: job.status, variant: 'warning' as const }
  }
}

function conflictSummary(conflicts: HarborJobConflict[]) {
  return conflicts.reduce(
    (acc, conflict) => {
      const key = conflict.action.toLowerCase()
      acc[key] = (acc[key] ?? 0) + 1
      return acc
    },
    {} as Record<string, number>,
  )
}

function triggerBlobDownload(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}

export function HarborJobDetailsSheet({ jobId, onOpenChange }: Props) {
  const realm = useActiveRealm()
  const [isDownloading, setIsDownloading] = useState(false)
  const { data, isLoading, isFetching, refetch } = useHarborJobDetails(jobId)

  const handleDownload = async () => {
    if (!realm || !jobId) return
    try {
      setIsDownloading(true)
      const artifact = await downloadHarborJobArtifact({ realm, jobId })
      triggerBlobDownload(artifact.blob, artifact.filename)
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to download artifact'
      toast.error(message)
    } finally {
      setIsDownloading(false)
    }
  }

  const job = data?.job
  const conflicts = data?.conflicts ?? []
  const badge = job ? statusBadge(job) : null
  const summary = conflictSummary(conflicts)

  return (
    <Sheet open={!!jobId} onOpenChange={onOpenChange}>
      <SheetContent className="w-[560px] overflow-y-auto sm:max-w-2xl">
        <SheetHeader>
          <SheetTitle className="text-left">Harbor Job Details</SheetTitle>
          <SheetDescription className="text-left">
            Inspect execution status, conflicts, and export artifacts for the selected Harbor job.
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          {isLoading ? (
            <div className="text-muted-foreground text-sm">Loading Harbor job details...</div>
          ) : null}

          {!isLoading && !job ? (
            <div className="text-muted-foreground text-sm">Select a Harbor job to inspect it.</div>
          ) : null}

          {job ? (
            <>
              <div className="flex items-start justify-between gap-4">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <h3 className="text-lg font-semibold capitalize">
                      {job.job_type} · {job.scope.replaceAll('_', ' ')}
                    </h3>
                    {badge ? <Badge variant={badge.variant}>{badge.label}</Badge> : null}
                  </div>
                  <p className="text-muted-foreground text-sm">
                    Created {new Date(job.created_at).toLocaleString()} · Updated{' '}
                    {formatRelativeTime(job.updated_at)}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="outline" size="sm" onClick={() => void refetch()}>
                    <RefreshCcw className={isFetching ? 'animate-spin' : ''} />
                    Refresh
                  </Button>
                  {data?.download_url ? (
                    <Button
                      size="sm"
                      onClick={() => void handleDownload()}
                      disabled={isDownloading}
                    >
                      <Download />
                      {isDownloading ? 'Downloading...' : 'Download'}
                    </Button>
                  ) : null}
                </div>
              </div>

              <div className="grid gap-3 sm:grid-cols-2">
                <div className="rounded-lg border p-4">
                  <p className="text-muted-foreground text-xs font-medium uppercase tracking-wide">
                    Progress
                  </p>
                  <p className="mt-2 text-2xl font-semibold">
                    {job.processed_resources} / {job.total_resources}
                  </p>
                  <p className="text-muted-foreground mt-1 text-xs">Processed resources</p>
                </div>
                <div className="rounded-lg border p-4">
                  <p className="text-muted-foreground text-xs font-medium uppercase tracking-wide">
                    Mutations
                  </p>
                  <p className="mt-2 text-2xl font-semibold">
                    {job.created_count + job.updated_count}
                  </p>
                  <p className="text-muted-foreground mt-1 text-xs">
                    {job.created_count} created · {job.updated_count} updated
                  </p>
                </div>
              </div>

              {job.error_message ? (
                <Alert variant="destructive">
                  <AlertTriangle className="h-4 w-4" />
                  <AlertTitle>Execution Error</AlertTitle>
                  <AlertDescription>{job.error_message}</AlertDescription>
                </Alert>
              ) : null}

              <div className="space-y-3">
                <div className="grid gap-3 sm:grid-cols-2">
                  <InfoBlock label="Job ID" value={job.id} mono />
                  <InfoBlock label="Conflict Policy" value={job.conflict_policy ?? 'Default'} />
                  <InfoBlock label="Dry Run" value={job.dry_run ? 'Yes' : 'No'} />
                  <InfoBlock
                    label="Completed At"
                    value={job.completed_at ? new Date(job.completed_at).toLocaleString() : '—'}
                  />
                </div>
              </div>

              <Separator />

              <div className="space-y-3">
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <h4 className="text-sm font-semibold">Conflict Log</h4>
                    <p className="text-muted-foreground text-xs">
                      Resource-level rename, skip, and overwrite decisions captured during import.
                    </p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {Object.entries(summary).map(([key, value]) => (
                      <Badge key={key} variant="outline">
                        {key}: {value}
                      </Badge>
                    ))}
                  </div>
                </div>

                {conflicts.length === 0 ? (
                  <div className="text-muted-foreground rounded-lg border border-dashed p-4 text-sm">
                    No conflicts were recorded for this job.
                  </div>
                ) : (
                  <div className="space-y-3">
                    {conflicts.map((conflict) => (
                      <div key={conflict.id} className="rounded-lg border p-4">
                        <div className="flex items-start justify-between gap-3">
                          <div className="space-y-1">
                            <div className="flex items-center gap-2">
                              <ShieldAlert className="text-muted-foreground h-4 w-4" />
                              <p className="font-medium">{conflict.resource_key}</p>
                            </div>
                            <p className="text-muted-foreground text-xs">
                              {new Date(conflict.created_at).toLocaleString()}
                            </p>
                          </div>
                          <Badge variant="outline">
                            {conflict.action} · {conflict.policy}
                          </Badge>
                        </div>

                        <div className="mt-3 grid gap-2 text-sm sm:grid-cols-2">
                          <InfoBlock
                            label="Original ID"
                            value={conflict.original_id ?? '—'}
                            mono
                          />
                          <InfoBlock
                            label="Resolved ID"
                            value={conflict.resolved_id ?? '—'}
                            mono
                          />
                        </div>

                        {conflict.message ? (
                          <p className="text-muted-foreground mt-3 text-sm">{conflict.message}</p>
                        ) : null}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </>
          ) : null}
        </div>
      </SheetContent>
    </Sheet>
  )
}

function InfoBlock({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="rounded-lg border p-3">
      <p className="text-muted-foreground text-xs font-medium uppercase tracking-wide">{label}</p>
      <p className={`mt-2 break-all text-sm ${mono ? 'font-mono' : ''}`}>{value}</p>
    </div>
  )
}
