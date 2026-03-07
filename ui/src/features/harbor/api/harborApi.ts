import { useSessionStore } from '@/entities/session/model/sessionStore'
import { refreshAccessToken } from '@/shared/api/client'

type HarborRequestConfig = RequestInit & {
  _isRetry?: boolean
  skipContentType?: boolean
}

export type HarborAsyncResponse = {
  job_id: string
  download_url?: string
}

export type HarborJob = {
  id: string
  realm_id: string
  job_type: string
  status: string
  scope: string
  total_resources: number
  processed_resources: number
  created_count: number
  updated_count: number
  dry_run: boolean
  conflict_policy?: string | null
  artifact_path?: string | null
  artifact_filename?: string | null
  artifact_content_type?: string | null
  error_message?: string | null
  created_at: string
  updated_at: string
  completed_at?: string | null
}

export type HarborJobConflict = {
  id: string
  job_id: string
  resource_key: string
  action: string
  policy: string
  original_id?: string | null
  resolved_id?: string | null
  message?: string | null
  created_at: string
}

export type HarborJobDetail = {
  job: HarborJob
  download_url?: string
}

export type HarborJobDetails = {
  job: HarborJob
  download_url?: string
  conflicts: HarborJobConflict[]
}

export type HarborImportResult = {
  dry_run: boolean
  resources: Array<{
    key: string
    status: string
    created: number
    updated: number
    errors?: string[]
    original_id?: string
    renamed_to?: string
  }>
  warnings?: string[]
}

export type HarborExportArchiveResult =
  | {
      mode: 'download'
      blob: Blob
      filename: string
    }
  | {
      mode: 'async'
      jobId: string
      downloadUrl?: string
    }

export type HarborImportResponse = HarborAsyncResponse | HarborImportResult

export function isHarborAsyncResponse(value: unknown): value is HarborAsyncResponse {
  return (
    !!value &&
    typeof value === 'object' &&
    'job_id' in value &&
    typeof (value as { job_id?: unknown }).job_id === 'string'
  )
}

export function summarizeImportResult(result: HarborImportResult) {
  return result.resources.reduce(
    (acc, resource) => {
      acc.created += resource.created ?? 0
      acc.updated += resource.updated ?? 0
      return acc
    },
    { created: 0, updated: 0 },
  )
}

export function isHarborJobActive(job: HarborJob) {
  return ['queued', 'processing', 'running', 'pending'].includes(job.status.toLowerCase())
}

async function harborRequest(url: string, config: HarborRequestConfig = {}): Promise<Response> {
  const token = useSessionStore.getState().accessToken
  const headers = new Headers(config.headers)

  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  if (!headers.has('Content-Type') && !config.skipContentType && typeof config.body === 'string') {
    headers.set('Content-Type', 'application/json')
  }

  const response = await fetch(url, {
    ...config,
    headers,
    credentials: 'include',
  })

  if (response.status === 401 && !config._isRetry) {
    try {
      const newToken = await refreshAccessToken()
      useSessionStore.getState().setSession(newToken)
      return harborRequest(url, { ...config, _isRetry: true })
    } catch {
      useSessionStore.getState().clearSession()
      throw new Error('Session expired')
    }
  }

  if (!response.ok) {
    const errorBody = await response.text()
    let errorMessage = `API Error: ${response.statusText}`
    try {
      const json = JSON.parse(errorBody)
      errorMessage = json.error || errorMessage
    } catch {
      /* ignore json parse error */
    }
    throw new Error(errorMessage)
  }

  return response
}

function extractFilename(response: Response, fallback: string) {
  const disposition = response.headers.get('content-disposition')
  if (!disposition) return fallback
  const match = /filename="([^"]+)"/.exec(disposition)
  return match?.[1] ?? fallback
}

export async function exportHarborArchive(params: {
  realm: string
  scope: string
  selection?: string[]
  includeSecrets?: boolean
  archiveFormat?: string
  asyncMode?: boolean
}): Promise<HarborExportArchiveResult> {
  const query = params.asyncMode !== undefined ? `?async=${params.asyncMode}` : ''
  const response = await harborRequest(`/api/realms/${params.realm}/harbor/export/archive${query}`, {
    method: 'POST',
    body: JSON.stringify({
      scope: params.scope,
      include_secrets: params.includeSecrets ?? false,
      selection: params.selection,
      archive_format: params.archiveFormat ?? 'zip',
    }),
  })

  const contentType = response.headers.get('content-type') ?? ''
  if (contentType.includes('application/json')) {
    const json = (await response.json()) as HarborAsyncResponse
    return {
      mode: 'async',
      jobId: json.job_id,
      downloadUrl: json.download_url,
    }
  }

  const blob = await response.blob()
  const filename = extractFilename(response, 'harbor-export.reauth')
  return {
    mode: 'download',
    blob,
    filename,
  }
}

export async function listHarborJobs(params: {
  realm: string
  limit?: number
}): Promise<HarborJob[]> {
  const query = new URLSearchParams()
  if (params.limit) query.set('limit', String(params.limit))
  const response = await harborRequest(
    `/api/realms/${params.realm}/harbor/jobs${query.toString() ? `?${query.toString()}` : ''}`,
    { method: 'GET' },
  )

  return (await response.json()) as HarborJob[]
}

export async function getHarborJobDetails(params: {
  realm: string
  jobId: string
}): Promise<HarborJobDetails> {
  const response = await harborRequest(
    `/api/realms/${params.realm}/harbor/jobs/${params.jobId}/details`,
    { method: 'GET' },
  )

  return (await response.json()) as HarborJobDetails
}

export async function downloadHarborJobArtifact(params: {
  realm: string
  jobId: string
}): Promise<{ blob: Blob; filename: string }> {
  const response = await harborRequest(
    `/api/realms/${params.realm}/harbor/jobs/${params.jobId}/download`,
    { method: 'GET' },
  )

  const blob = await response.blob()
  const filename = extractFilename(response, 'harbor-export.reauth')
  return { blob, filename }
}

export async function importHarborArchive(params: {
  realm: string
  scope: string
  id?: string
  file: File
  conflictPolicy: string
  dryRun: boolean
  asyncMode?: boolean
}): Promise<HarborImportResponse> {
  const query = new URLSearchParams()
  if (params.dryRun) query.set('dry_run', 'true')
  if (params.asyncMode !== undefined) query.set('async', params.asyncMode ? 'true' : 'false')
  const suffix = query.toString()

  const form = new FormData()
  form.append('bundle', params.file)
  form.append('scope', params.scope)
  if (params.id) form.append('id', params.id)
  form.append('conflict_policy', params.conflictPolicy)
  form.append('dry_run', String(params.dryRun))

  const response = await harborRequest(
    `/api/realms/${params.realm}/harbor/import/archive${suffix ? `?${suffix}` : ''}`,
    {
      method: 'POST',
      body: form,
      skipContentType: true,
    },
  )

  return (await response.json()) as HarborImportResponse
}

export async function importHarborBundle(params: {
  realm: string
  scope: string
  id?: string
  bundle: unknown
  conflictPolicy: string
  dryRun: boolean
  asyncMode?: boolean
}): Promise<HarborImportResponse> {
  const query = new URLSearchParams()
  if (params.dryRun) query.set('dry_run', 'true')
  if (params.asyncMode !== undefined) query.set('async', params.asyncMode ? 'true' : 'false')
  const suffix = query.toString()

  const response = await harborRequest(
    `/api/realms/${params.realm}/harbor/import${suffix ? `?${suffix}` : ''}`,
    {
      method: 'POST',
      body: JSON.stringify({
        scope: params.scope,
        id: params.id,
        bundle: params.bundle,
        conflict_policy: params.conflictPolicy,
        dry_run: params.dryRun,
      }),
    },
  )

  return (await response.json()) as HarborImportResponse
}
