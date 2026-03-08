import { useEffect, useMemo, useState } from 'react'

import {
  AlertTriangle,
  Download,
  Eye,
  History,
  Package,
  RefreshCcw,
  UploadCloud,
} from 'lucide-react'
import { toast } from 'sonner'
import { useSearchParams } from 'react-router-dom'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Checkbox } from '@/components/checkbox'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/table'
import {
  type HarborExportArchiveResult,
  type HarborJob,
  isHarborJobActive,
} from '@/features/harbor/api/harborApi'
import { useHarborExportArchive } from '@/features/harbor/api/useHarborExportArchive'
import { useHarborImportBundle } from '@/features/harbor/api/useHarborImportBundle'
import { useHarborJobs } from '@/features/harbor/api/useHarborJobs'
import { HarborJobDetailsSheet } from '@/features/harbor/components/HarborJobDetailsSheet'
import { cn, formatRelativeTime } from '@/lib/utils'
import { Main } from '@/widgets/Layout/Main'

const RESOURCE_OPTIONS: Array<{
  id: string
  label: string
  disabled?: boolean
  badge?: string
}> = [
  { id: 'all_settings', label: 'All Settings' },
  { id: 'realm', label: 'Realm Settings' },
  { id: 'themes', label: 'Themes' },
  { id: 'clients', label: 'Clients' },
  { id: 'flows', label: 'Auth Flows' },
  { id: 'users', label: 'Users' },
  { id: 'roles', label: 'Roles' },
]

const EMPTY_JOBS: HarborJob[] = []

function getJobBadge(job: HarborJob) {
  const status = job.status.toLowerCase()
  if (status === 'completed') return { label: 'Completed', variant: 'success' as const }
  if (status === 'failed' || status === 'expired') {
    return { label: job.status, variant: 'destructive' as const }
  }
  if (status === 'queued' || status === 'pending') {
    return { label: job.status, variant: 'secondary' as const }
  }
  return { label: job.status, variant: 'warning' as const }
}

function formatJobType(job: HarborJob) {
  const prefix = job.job_type === 'export' ? 'Export' : 'Import'
  return `${prefix} · ${job.scope.replaceAll('_', ' ')}`
}

function formatItemsProcessed(job: HarborJob) {
  const total = Math.max(job.total_resources, 0)
  if (total === 0) return '0 / 0'
  return `${job.processed_resources} / ${total}`
}

function jobOutcomeBadges(job: HarborJob) {
  const badges: Array<{ label: string; variant: 'success' | 'secondary' | 'warning' | 'info' }> = []

  if (job.created_count > 0) {
    badges.push({ label: `${job.created_count} created`, variant: 'success' })
  }
  if (job.updated_count > 0) {
    badges.push({ label: `${job.updated_count} updated`, variant: 'info' })
  }
  if (job.dry_run) {
    badges.push({ label: 'Validation only', variant: 'secondary' })
  }
  if (job.status.toLowerCase() === 'completed' && job.created_count === 0 && job.updated_count === 0 && !job.dry_run) {
    badges.push({ label: 'No writes', variant: 'secondary' })
  }

  return badges
}

function jobStatusHint(job: HarborJob) {
  if (job.error_message) {
    return job.error_message
  }
  if (job.status.toLowerCase() === 'completed') {
    if (job.dry_run) return 'Validation completed successfully'
    if (job.created_count > 0 || job.updated_count > 0) return 'Changes applied successfully'
    return 'Completed without resource changes'
  }
  if (isHarborJobActive(job)) {
    return 'Harbor is still processing this job'
  }
  return null
}

function progressPercentage(job: HarborJob) {
  if (job.total_resources <= 0) return 0
  return Math.max(0, Math.min(100, Math.round((job.processed_resources / job.total_resources) * 100)))
}

function ProgressBar({ value }: { value: number }) {
  return (
    <div className="bg-muted h-2 w-full overflow-hidden rounded-full">
      <div
        className="bg-primary h-full rounded-full transition-[width] duration-300"
        style={{ width: `${value}%` }}
      />
    </div>
  )
}

export function HarborDashboardPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [includeSecrets, setIncludeSecrets] = useState(false)
  const [dryRun, setDryRun] = useState(true)
  const [conflictPolicy, setConflictPolicy] = useState('skip')
  const [bundleFile, setBundleFile] = useState<File | null>(null)
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null)
  const [manifestPreview, setManifestPreview] = useState<{
    version?: string
    exportedAt?: string
    sourceRealm?: string
  } | null>(null)
  const [resourceSelection, setResourceSelection] = useState<Record<string, boolean>>({
    all_settings: false,
    realm: true,
    themes: true,
    clients: true,
    flows: true,
    users: false,
    roles: true,
  })

  const exportMutation = useHarborExportArchive()
  const importMutation = useHarborImportBundle()
  const jobsQuery = useHarborJobs(20)
  const requestedJobId = searchParams.get('job')
  const source = searchParams.get('source')

  const manifest = manifestPreview
    ? manifestPreview
    : bundleFile
      ? {
          version: '—',
          exportedAt: new Date(bundleFile.lastModified).toLocaleString(),
          sourceRealm: '—',
        }
      : null

  const exportSelection = resourceSelection.all_settings
    ? ['realm', 'theme', 'client', 'flow', 'user', 'role']
    : (['realm', 'themes', 'clients', 'flows', 'users', 'roles'] as const)
        .filter((key) => resourceSelection[key])
        .map((key) =>
          key === 'realm'
            ? 'realm'
            : key === 'themes'
              ? 'theme'
            : key === 'clients'
              ? 'client'
            : key === 'users'
              ? 'user'
                : key === 'roles'
                  ? 'role'
                  : 'flow',
        )

  const liveJobs = jobsQuery.data ?? EMPTY_JOBS
  const activeJobs = liveJobs.filter((job) => isHarborJobActive(job))
  const openedFromContextualAction = source === 'contextual'

  useEffect(() => {
    if (requestedJobId) {
      setSelectedJobId(requestedJobId)
    }
  }, [requestedJobId])

  const contextualJob = useMemo(
    () => liveJobs.find((job) => job.id === requestedJobId),
    [liveJobs, requestedJobId],
  )

  const onToggleResource = (id: string, checked: boolean | 'indeterminate') => {
    setResourceSelection((prev) => {
      const next = { ...prev, [id]: checked === true }
      if (id === 'all_settings') {
        next.realm = checked === true
        next.themes = checked === true
        next.clients = checked === true
        next.flows = checked === true
        next.users = checked === true
        next.roles = checked === true
      }
      if (['realm', 'themes', 'clients', 'flows', 'users', 'roles'].includes(id) && !checked) {
        next.all_settings = false
      }
      if (['realm', 'themes', 'clients', 'flows', 'users', 'roles'].includes(id) && checked) {
        next.all_settings =
          next.realm && next.themes && next.clients && next.flows && next.users && next.roles
      }
      return next
    })
  }

  const handleExport = async () => {
    if (exportSelection.length === 0) {
      toast.error('Select at least one resource to export')
      return
    }

    try {
      const result = await exportMutation.mutateAsync({
        scope: 'full_realm',
        selection: exportSelection,
        includeSecrets,
        archiveFormat: 'zip',
      })

      if (result.mode === 'async') {
        toast.message('Export queued', {
          description: `Job ${result.jobId} is processing.`,
        })
        setSelectedJobId(result.jobId)
        return
      }

      triggerDownload(result)
      toast.success('Export ready', {
        description: 'Your Harbor bundle download has started.',
      })
    } catch {
      // handled via mutation onError
    }
  }

  const handleImport = async () => {
    if (!bundleFile) {
      toast.error('Select a bundle to import')
      return
    }

    try {
      const result = await importMutation.mutateAsync({
        file: bundleFile,
        scope: 'full_realm',
        conflictPolicy,
        dryRun,
      })

      if ('job_id' in result) {
        setSelectedJobId(result.job_id)
      }
    } catch {
      // handled via mutation onError
    }
  }

  const triggerDownload = (result: HarborExportArchiveResult) => {
    if (result.mode !== 'download') return
    const url = URL.createObjectURL(result.blob)
    const link = document.createElement('a')
    link.href = url
    link.download = result.filename
    document.body.appendChild(link)
    link.click()
    link.remove()
    URL.revokeObjectURL(url)
  }

  return (
    <>
      <Main fixed className="flex h-full flex-col gap-6 p-6 lg:p-10 overflow-auto">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">
            Harbor Management Hub
          </h1>
          <p className="text-muted-foreground text-sm">
            Package and move realm resources across environments with import and export bundles.
          </p>
        </div>

        <div className="grid gap-6 lg:grid-cols-2">
          <Card className="flex flex-col">
            <CardHeader className="flex flex-row items-start justify-between space-y-0">
              <div className="space-y-1">
                <CardTitle className="text-lg">Export Workspace</CardTitle>
                <CardDescription>Select what to include in the bundle.</CardDescription>
              </div>
              <div className="bg-primary/10 text-primary flex h-9 w-9 items-center justify-center rounded-md">
                <Download className="h-4 w-4" />
              </div>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col gap-6">
              <div className="space-y-3">
                <p className="text-muted-foreground text-xs font-semibold uppercase tracking-wider">
                  Resource Selection
                </p>
                <div className="grid gap-3 sm:grid-cols-2">
                  {RESOURCE_OPTIONS.map((option) => (
                    <label
                      key={option.id}
                      className={cn(
                        'flex items-center gap-3 rounded-lg border p-3 transition-colors',
                        option.disabled
                          ? 'text-muted-foreground/70 cursor-not-allowed opacity-70'
                          : 'hover:bg-muted/50 cursor-pointer',
                      )}
                    >
                      <Checkbox
                        checked={resourceSelection[option.id]}
                        onCheckedChange={(checked) => onToggleResource(option.id, checked)}
                        disabled={option.disabled}
                      />
                      <span className="text-sm font-medium">{option.label}</span>
                      {option.badge ? (
                        <Badge variant="outline" className="ml-auto text-[10px] uppercase">
                          {option.badge}
                        </Badge>
                      ) : null}
                    </label>
                  ))}
                </div>
              </div>

              <Alert className="bg-muted/40 border-muted-foreground/30">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle className="flex items-center justify-between gap-3">
                  <span>Include Secrets</span>
                  <Switch checked={includeSecrets} onCheckedChange={setIncludeSecrets} />
                </AlertTitle>
                <AlertDescription className="text-muted-foreground text-xs">
                  Enabling this includes client secrets and private keys. Store the bundle
                  securely.
                </AlertDescription>
              </Alert>

              <Button
                className="mt-auto w-full"
                size="lg"
                onClick={() => void handleExport()}
                disabled={exportMutation.isPending}
              >
                <Package className="h-4 w-4" />
                {exportMutation.isPending ? 'Generating Bundle...' : 'Generate .reauth Bundle'}
              </Button>
            </CardContent>
          </Card>

          <Card className="flex flex-col">
            <CardHeader className="flex flex-row items-start justify-between space-y-0">
              <div className="space-y-1">
                <CardTitle className="text-lg">Import Workspace</CardTitle>
                <CardDescription>Upload a bundle and validate before applying.</CardDescription>
              </div>
              <div className="bg-primary/10 text-primary flex h-9 w-9 items-center justify-center rounded-md">
                <UploadCloud className="h-4 w-4" />
              </div>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col gap-6">
              <label
                className={cn(
                  'flex cursor-pointer flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed p-6 text-center transition-colors',
                  'hover:border-primary/40 hover:bg-muted/40',
                )}
              >
                <Input
                  type="file"
                  accept=".reauth,.json"
                  className="hidden"
                  onChange={async (event) => {
                    const file = event.target.files?.[0] ?? null
                    setBundleFile(file)
                    if (!file) {
                      setManifestPreview(null)
                      return
                    }

                    if (file.name.toLowerCase().endsWith('.json')) {
                      try {
                        const raw = await file.text()
                        const parsed = JSON.parse(raw) as {
                          manifest?: {
                            version?: string
                            exported_at?: string
                            source_realm?: string
                          }
                        }
                        setManifestPreview({
                          version: parsed.manifest?.version,
                          exportedAt: parsed.manifest?.exported_at,
                          sourceRealm: parsed.manifest?.source_realm,
                        })
                      } catch {
                        setManifestPreview(null)
                      }
                    } else {
                      setManifestPreview(null)
                    }
                  }}
                />
                <div className="bg-primary/10 text-primary flex h-11 w-11 items-center justify-center rounded-full">
                  <UploadCloud className="h-5 w-5" />
                </div>
                <div className="space-y-1">
                  <p className="text-sm font-semibold">Drag &amp; drop your bundle here</p>
                  <p className="text-muted-foreground text-xs">
                    Accepts .reauth or .json files up to 50MB
                  </p>
                </div>
                {bundleFile ? (
                  <Badge variant="info" className="mt-1">
                    {bundleFile.name}
                  </Badge>
                ) : (
                  <Badge variant="muted" className="mt-1">
                    No file selected
                  </Badge>
                )}
              </label>

              <div className="space-y-3 rounded-lg border p-4">
                <div className="flex items-center justify-between">
                  <p className="text-muted-foreground text-xs font-semibold uppercase tracking-wider">
                    Bundle Manifest
                  </p>
                  <Badge variant={manifest ? 'success' : 'muted'}>
                    {manifest ? 'READY' : 'AWAITING'}
                  </Badge>
                </div>
                <div className="text-sm">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Version</span>
                    <span className="font-medium">{manifest?.version ?? '—'}</span>
                  </div>
                  <div className="mt-2 flex items-center justify-between">
                    <span className="text-muted-foreground">Source Realm</span>
                    <span className="font-medium">{manifest?.sourceRealm ?? '—'}</span>
                  </div>
                  <div className="mt-2 flex items-center justify-between">
                    <span className="text-muted-foreground">Exported At</span>
                    <span className="font-medium">{manifest?.exportedAt ?? '—'}</span>
                  </div>
                </div>
              </div>

              <div className="grid gap-4">
                <div className="space-y-2">
                  <Label>Conflict Resolution Strategy</Label>
                  <Select value={conflictPolicy} onValueChange={setConflictPolicy}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select policy" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="skip">Skip (Keep Existing)</SelectItem>
                      <SelectItem value="overwrite">Overwrite (Replace with Bundle)</SelectItem>
                      <SelectItem value="rename">Rename (Create Duplicate)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="bg-muted/40 flex items-center justify-between rounded-lg border p-3">
                  <div>
                    <p className="text-sm font-medium">Validate Only</p>
                    <p className="text-muted-foreground text-xs">
                      Run a dry-run check without applying changes.
                    </p>
                  </div>
                  <Switch checked={dryRun} onCheckedChange={setDryRun} />
                </div>
              </div>

              <Button
                className="mt-auto w-full"
                variant="secondary"
                onClick={() => void handleImport()}
                disabled={importMutation.isPending || !bundleFile}
              >
                {importMutation.isPending ? 'Importing...' : 'Start Import Process'}
              </Button>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="bg-primary/10 text-primary flex h-9 w-9 items-center justify-center rounded-md">
                <History className="h-4 w-4" />
              </div>
              <div>
                <CardTitle className="text-lg">Recent Harbor Jobs</CardTitle>
                <CardDescription>
                  Latest import/export activity across this realm. Active jobs poll automatically.
                </CardDescription>
              </div>
            </div>
            <Button variant="ghost" size="sm" onClick={() => void jobsQuery.refetch()}>
              <RefreshCcw className={jobsQuery.isFetching ? 'animate-spin' : ''} />
              Refresh
            </Button>
          </CardHeader>
          <CardContent className="pt-0">
            {jobsQuery.isError ? (
              <Alert variant="destructive" className="mb-4">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>Unable to load Harbor jobs</AlertTitle>
                <AlertDescription>
                  {jobsQuery.error instanceof Error
                    ? jobsQuery.error.message
                    : 'Unknown Harbor jobs error'}
                </AlertDescription>
              </Alert>
            ) : null}

            <div className="mb-4 flex flex-wrap gap-2">
              <Badge variant="outline">{liveJobs.length} jobs tracked</Badge>
              <Badge variant={activeJobs.length > 0 ? 'warning' : 'muted'}>
                {activeJobs.length} active
              </Badge>
              <Badge variant={activeJobs.length > 0 ? 'info' : 'muted'}>
                {activeJobs.length > 0 ? 'Polling every 2s' : 'Polling every 10s'}
              </Badge>
            </div>

            {openedFromContextualAction ? (
              <Alert className="mb-4">
                <Eye className="h-4 w-4" />
                <AlertTitle>Opened from contextual Harbor action</AlertTitle>
                <AlertDescription>
                  {contextualJob
                    ? `Viewing job ${contextualJob.id.slice(0, 8)} for ${formatJobType(contextualJob)}.`
                    : requestedJobId
                      ? `Viewing Harbor job ${requestedJobId.slice(0, 8)}.`
                      : 'Viewing a Harbor job from a contextual action.'}
                </AlertDescription>
              </Alert>
            ) : null}

            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Job Type</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Outcome</TableHead>
                  <TableHead>Items Processed</TableHead>
                  <TableHead>Date</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {jobsQuery.isLoading ? (
                  <TableRow>
                    <TableCell colSpan={6} className="h-24">
                      <div className="flex items-center justify-center gap-3 text-sm">
                        <RefreshCcw className="text-muted-foreground h-4 w-4 animate-spin" />
                        <span className="text-muted-foreground">
                          Loading Harbor jobs and live progress…
                        </span>
                      </div>
                    </TableCell>
                  </TableRow>
                ) : liveJobs.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-muted-foreground h-24 text-center">
                      No Harbor jobs yet. Imports and async exports will appear here.
                    </TableCell>
                  </TableRow>
                ) : (
                  liveJobs.map((job) => {
                    const badge = getJobBadge(job)
                    const outcome = jobOutcomeBadges(job)
                    const hint = jobStatusHint(job)
                    const progress = progressPercentage(job)
                    return (
                      <TableRow key={job.id}>
                        <TableCell>
                          <div className="space-y-1">
                            <div className="font-medium">{formatJobType(job)}</div>
                            <div className="text-muted-foreground text-xs">
                              {job.dry_run ? 'Dry run' : 'Write mode'}
                            </div>
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="space-y-1">
                            <Badge variant={badge.variant}>{badge.label}</Badge>
                            {hint ? (
                              <div className="text-muted-foreground max-w-xs text-xs">
                                {hint}
                              </div>
                            ) : null}
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex max-w-xs flex-wrap gap-2">
                            {outcome.length > 0 ? (
                              outcome.map((entry) => (
                                <Badge key={entry.label} variant={entry.variant}>
                                  {entry.label}
                                </Badge>
                              ))
                            ) : (
                              <Badge variant="muted">Pending outcome</Badge>
                            )}
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="space-y-1">
                            <div>{formatItemsProcessed(job)}</div>
                            <div className="text-muted-foreground text-xs">
                              {job.created_count} created · {job.updated_count} updated
                            </div>
                            {isHarborJobActive(job) ? (
                              <div className="pt-1">
                                <div className="mb-1 flex items-center justify-between text-[11px]">
                                  <span className="text-muted-foreground">Progress</span>
                                  <span className="font-medium">{progress}%</span>
                                </div>
                                <ProgressBar value={progress} />
                              </div>
                            ) : null}
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="space-y-1">
                            <div>{new Date(job.created_at).toLocaleString()}</div>
                            <div className="text-muted-foreground text-xs">
                              {formatRelativeTime(job.updated_at)}
                            </div>
                          </div>
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-2">
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => setSelectedJobId(job.id)}
                            >
                              <Eye />
                              Details
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    )
                  })
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </Main>

      <HarborJobDetailsSheet
        jobId={selectedJobId}
        onOpenChange={(open) => {
          if (!open) {
            setSelectedJobId(null)
            if (searchParams.has('job') || searchParams.has('source')) {
              const next = new URLSearchParams(searchParams)
              next.delete('job')
              next.delete('source')
              setSearchParams(next, { replace: true })
            }
          }
        }}
      />
    </>
  )
}

export default HarborDashboardPage
