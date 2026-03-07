import { useState } from 'react'

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

const RESOURCE_OPTIONS = [
  { id: 'all_settings', label: 'All Settings' },
  { id: 'themes', label: 'Themes' },
  { id: 'clients', label: 'Clients' },
  { id: 'flows', label: 'Auth Flows' },
  { id: 'roles', label: 'Roles', disabled: true, badge: 'Soon' },
]

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

export function HarborDashboardPage() {
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
    themes: true,
    clients: true,
    flows: true,
    roles: false,
  })

  const exportMutation = useHarborExportArchive()
  const importMutation = useHarborImportBundle()
  const jobsQuery = useHarborJobs(20)

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
    ? ['theme', 'client', 'flow']
    : (['themes', 'clients', 'flows'] as const)
        .filter((key) => resourceSelection[key])
        .map((key) => (key === 'themes' ? 'theme' : key === 'clients' ? 'client' : 'flow'))

  const liveJobs = jobsQuery.data ?? []
  const activeJobs = liveJobs.filter((job) => isHarborJobActive(job))

  const onToggleResource = (id: string, checked: boolean | 'indeterminate') => {
    setResourceSelection((prev) => {
      const next = { ...prev, [id]: checked === true }
      if (id === 'all_settings') {
        next.themes = checked === true
        next.clients = checked === true
        next.flows = checked === true
      }
      if (['themes', 'clients', 'flows'].includes(id) && !checked) {
        next.all_settings = false
      }
      if (['themes', 'clients', 'flows'].includes(id) && checked) {
        next.all_settings = next.themes && next.clients && next.flows
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
            </div>

            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Job Type</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Items Processed</TableHead>
                  <TableHead>Date</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {liveJobs.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="text-muted-foreground h-24 text-center">
                      No Harbor jobs yet. Imports and async exports will appear here.
                    </TableCell>
                  </TableRow>
                ) : (
                  liveJobs.map((job) => {
                    const badge = getJobBadge(job)
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
                          <Badge variant={badge.variant}>{badge.label}</Badge>
                        </TableCell>
                        <TableCell>
                          <div className="space-y-1">
                            <div>{formatItemsProcessed(job)}</div>
                            <div className="text-muted-foreground text-xs">
                              {job.created_count} created · {job.updated_count} updated
                            </div>
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
          if (!open) setSelectedJobId(null)
        }}
      />
    </>
  )
}

export default HarborDashboardPage
