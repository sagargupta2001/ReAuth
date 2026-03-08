import { useRef, useState } from 'react'

import { useQueryClient, type QueryKey } from '@tanstack/react-query'
import { Download, Loader2, Upload } from 'lucide-react'
import { toast } from 'sonner'

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
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import {
  isHarborAsyncResponse,
  type HarborExportArchiveResult,
  type HarborImportResult,
} from '@/features/harbor/api/harborApi'
import { useHarborExportArchive } from '@/features/harbor/api/useHarborExportArchive'
import { useHarborImportBundle } from '@/features/harbor/api/useHarborImportBundle'

type ConflictPolicy = 'overwrite' | 'skip' | 'rename'

const DEFAULT_POLICIES: ConflictPolicy[] = ['overwrite', 'skip']

function triggerDownload(result: HarborExportArchiveResult) {
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

type Props = {
  scope: 'theme' | 'client' | 'flow'
  id: string
  resourceLabel: string
  invalidateKeys?: QueryKey[]
  allowedConflictPolicies?: ConflictPolicy[]
  size?: 'sm' | 'default'
  onImportSuccess?: (result: HarborImportResult) => void
}

export function HarborResourceActions({
  scope,
  id,
  resourceLabel,
  invalidateKeys = [],
  allowedConflictPolicies = DEFAULT_POLICIES,
  size = 'sm',
  onImportSuccess,
}: Props) {
  const queryClient = useQueryClient()
  const navigate = useRealmNavigate()
  const exportMutation = useHarborExportArchive()
  const importMutation = useHarborImportBundle()
  const fileRef = useRef<HTMLInputElement | null>(null)
  const [isImportOpen, setIsImportOpen] = useState(false)
  const [bundleFile, setBundleFile] = useState<File | null>(null)
  const [dryRun, setDryRun] = useState(false)
  const [conflictPolicy, setConflictPolicy] = useState<ConflictPolicy>(
    allowedConflictPolicies[0] ?? 'overwrite',
  )

  const isBusy = exportMutation.isPending || importMutation.isPending

  const invalidateResourceQueries = async () => {
    await Promise.all(
      invalidateKeys.map((queryKey) => queryClient.invalidateQueries({ queryKey })),
    )
  }

  const handleExport = async () => {
    try {
      const result = await exportMutation.mutateAsync({
        scope,
        id,
        includeSecrets: false,
        archiveFormat: 'zip',
      })

      if (result.mode === 'async') {
        toast.message('Export queued', {
          description: `${resourceLabel} export is processing in Harbor.`,
        })
        navigate(`/harbor?job=${result.jobId}&source=contextual`)
        return
      }

      triggerDownload(result)
      toast.success('Export ready', {
        description: `${resourceLabel} bundle download has started.`,
      })
    } catch {
      // handled by hook
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
        scope,
        id,
        conflictPolicy,
        dryRun,
      })

      if (!isHarborAsyncResponse(result)) {
        await invalidateResourceQueries()
        onImportSuccess?.(result)
      } else {
        navigate(`/harbor?job=${result.job_id}&source=contextual`)
      }

      setIsImportOpen(false)
      setBundleFile(null)
      if (fileRef.current) {
        fileRef.current.value = ''
      }
    } catch {
      // handled by hook
    }
  }

  return (
    <>
      <div className="flex items-center gap-2">
        <Button variant="outline" size={size} onClick={() => void handleExport()} disabled={isBusy}>
          {exportMutation.isPending ? (
            <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
          ) : (
            <Download className="mr-2 h-3.5 w-3.5" />
          )}
          Export
        </Button>
        <Button variant="outline" size={size} onClick={() => setIsImportOpen(true)} disabled={isBusy}>
          {importMutation.isPending ? (
            <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
          ) : (
            <Upload className="mr-2 h-3.5 w-3.5" />
          )}
          Import
        </Button>
      </div>

      <Dialog open={isImportOpen} onOpenChange={setIsImportOpen}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>Import {resourceLabel}</DialogTitle>
            <DialogDescription>
              This targets the current {scope}. Use Harbor Hub for full-bundle or cross-resource
              imports.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-5">
            <div className="space-y-2">
              <Label htmlFor={`harbor-import-${scope}-${id}`}>Bundle file</Label>
              <Input
                id={`harbor-import-${scope}-${id}`}
                ref={fileRef}
                type="file"
                accept=".reauth,.json"
                onChange={(event) => setBundleFile(event.target.files?.[0] ?? null)}
              />
              <p className="text-muted-foreground text-xs">
                Accepts `.reauth` and `.json` Harbor bundles.
              </p>
            </div>

            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <Label>Conflict policy</Label>
                <Select
                  value={conflictPolicy}
                  onValueChange={(value) => setConflictPolicy(value as ConflictPolicy)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select a policy" />
                  </SelectTrigger>
                  <SelectContent>
                    {allowedConflictPolicies.map((policy) => (
                      <SelectItem key={policy} value={policy}>
                        {policy.charAt(0).toUpperCase() + policy.slice(1)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="rounded-lg border p-3">
                <div className="flex items-center justify-between gap-3">
                  <div className="space-y-1">
                    <p className="text-sm font-medium">Validate only</p>
                    <p className="text-muted-foreground text-xs">
                      Run a dry-run without writing changes.
                    </p>
                  </div>
                  <Switch checked={dryRun} onCheckedChange={setDryRun} />
                </div>
              </div>
            </div>

            <div className="bg-muted/40 flex items-start gap-2 rounded-lg border p-3 text-xs">
              <Badge variant="outline" className="mt-0.5">
                Harbor
              </Badge>
              <p className="text-muted-foreground">
                Resource-level imports are best for replacing or validating the current resource.
              </p>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsImportOpen(false)}>
              Cancel
            </Button>
            <Button onClick={() => void handleImport()} disabled={!bundleFile || importMutation.isPending}>
              {importMutation.isPending ? 'Importing...' : dryRun ? 'Run Validation' : 'Import Bundle'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
