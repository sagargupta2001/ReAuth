import * as React from 'react'
import CodeMirror from '@uiw/react-codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { oneDark } from '@codemirror/theme-one-dark'
import { autocompletion, type CompletionContext } from '@codemirror/autocomplete'
import { linter, type Diagnostic } from '@codemirror/lint'

import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'
import { Button } from '@/components/button'
import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import { FluidCanvas } from '@/features/fluid/components/FluidCanvas'
import { useTheme } from '@/app/providers/ThemeContext'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/command'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip'
import { CircleHelp } from 'lucide-react'

const PATCH_NODE_TYPES = ['Box', 'Text', 'Image', 'Icon', 'Input', 'Component'] as const
const PATCH_COMPONENT_TYPES = ['Button', 'Divider', 'Input', 'Link', 'SocialProvider'] as const
const PATCH_PROPERTY_KEYS = [
  'id',
  'type',
  'component',
  'props',
  'layout',
  'size',
  'children',
  'slots',
] as const

type PatchNode = {
  id?: string
  type?: string
  component?: string
  children?: PatchNode[]
  slots?: Record<string, PatchNode>
}

type PatchValidation = {
  nodes: ThemeNode[] | null
  errors: string[]
}

function parsePatchPayload(raw: string): PatchValidation {
  if (!raw.trim()) {
    return { nodes: null, errors: [] }
  }
  try {
    const parsed = JSON.parse(raw)
    if (Array.isArray(parsed)) {
      return { nodes: parsed as ThemeNode[], errors: [] }
    }
    if (parsed && typeof parsed === 'object' && Array.isArray((parsed as { nodes?: unknown }).nodes)) {
      return { nodes: (parsed as { nodes: ThemeNode[] }).nodes, errors: [] }
    }
    return { nodes: null, errors: ['Patch JSON must be an array or { "nodes": [...] } object.'] }
  } catch (err) {
    return {
      nodes: null,
      errors: [err instanceof Error ? err.message : 'Invalid JSON'],
    }
  }
}

function validatePatchNodes(nodes: PatchNode[], errors: string[], path: string) {
  nodes.forEach((node, index) => {
    const nodePath = `${path}[${index}]`
    if (!node || typeof node !== 'object') {
      errors.push(`${nodePath} must be an object`)
      return
    }
    const type = node.type
    if (!type || !PATCH_NODE_TYPES.includes(type as typeof PATCH_NODE_TYPES[number])) {
      errors.push(`${nodePath}.type must be one of: ${PATCH_NODE_TYPES.join(', ')}`)
      return
    }
    if (type === 'Component') {
      const component = node.component
      if (!component || !PATCH_COMPONENT_TYPES.includes(component as typeof PATCH_COMPONENT_TYPES[number])) {
        errors.push(
          `${nodePath}.component must be one of: ${PATCH_COMPONENT_TYPES.join(', ')}`,
        )
      }
    }
    if (Array.isArray(node.children)) {
      validatePatchNodes(node.children, errors, `${nodePath}.children`)
    }
    if (node.slots && typeof node.slots === 'object') {
      Object.entries(node.slots).forEach(([key, value]) => {
        validatePatchNodes([value as PatchNode], errors, `${nodePath}.slots.${key}`)
      })
    }
  })
}

type DiffLine = { type: 'equal' | 'add' | 'del'; text: string }

function diffLines(a: string[], b: string[]): DiffLine[] {
  const n = a.length
  const m = b.length
  const dp = Array.from({ length: n + 1 }, () => Array(m + 1).fill(0))
  for (let i = n - 1; i >= 0; i -= 1) {
    for (let j = m - 1; j >= 0; j -= 1) {
      if (a[i] === b[j]) {
        dp[i][j] = dp[i + 1][j + 1] + 1
      } else {
        dp[i][j] = Math.max(dp[i + 1][j], dp[i][j + 1])
      }
    }
  }

  const result: DiffLine[] = []
  let i = 0
  let j = 0
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      result.push({ type: 'equal', text: a[i] })
      i += 1
      j += 1
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      result.push({ type: 'del', text: a[i] })
      i += 1
    } else {
      result.push({ type: 'add', text: b[j] })
      j += 1
    }
  }
  while (i < n) {
    result.push({ type: 'del', text: a[i] })
    i += 1
  }
  while (j < m) {
    result.push({ type: 'add', text: b[j] })
    j += 1
  }
  return result
}

interface AutoFormProps {
  schema: Record<string, unknown>
  values: Record<string, unknown>
  onChange: (newValues: Record<string, unknown>) => void
  codeSuggestions?: Record<string, string[]>
  codePreviewTheme?: {
    tokens: Record<string, unknown>
    layout: Record<string, unknown>
    assets: ThemeAsset[]
    nodes?: ThemeNode[]
  }
  codeEditorMeta?: {
    currentTemplateKey?: string | null
  }
}

export function AutoForm({
  schema,
  values = {},
  onChange,
  codeSuggestions,
  codePreviewTheme,
  codeEditorMeta,
}: AutoFormProps) {
  if (!schema || !schema.properties) return null

  const properties = schema.properties as Record<string, Record<string, unknown>>

  const handleChange = (key: string, value: unknown) => {
    onChange({
      ...values,
      [key]: value,
    })
  }

  return (
    <div className="grid gap-4">
      {Object.entries(properties).map(([key, fieldSchema]) => {
        const value = values[key] ?? fieldSchema.default
        const error = null // todo: Integrate Zod validation errors here

        return (
          <FieldRenderer
            key={key}
            name={key}
            schema={fieldSchema}
            value={value}
            onChange={(val) => handleChange(key, val)}
            suggestions={codeSuggestions?.[key]}
            previewTheme={codePreviewTheme}
            editorMeta={codeEditorMeta}
            error={error}
          />
        )
      })}
    </div>
  )
}

/**
 * Dispatches the correct input component based on the schema type
 */
function FieldRenderer({
  name,
  schema,
  value,
  onChange,
  suggestions = [],
  previewTheme,
  editorMeta,
}: {
  name: string
  schema: Record<string, unknown>
  value: unknown
  onChange: (val: unknown) => void
  error?: string | null
  suggestions?: string[]
  previewTheme?: AutoFormProps['codePreviewTheme']
  editorMeta?: AutoFormProps['codeEditorMeta']
}) {
  const id = React.useId()
  const label = (schema.title as string) || name
  const description = schema.description as string | undefined

  // 1. ENUM (Select)
  if (schema.enum && Array.isArray(schema.enum)) {
    return (
      <div className="space-y-2">
        <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
        <Select value={value as string} onValueChange={onChange}>
          <SelectTrigger id={id} className="h-8 text-xs">
            <SelectValue placeholder="Select..." />
          </SelectTrigger>
          <SelectContent>
            {schema.enum.map((option: string) => (
              <SelectItem key={option} value={option} className="text-xs">
                {option}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
      </div>
    )
  }

  // 2. BOOLEAN (Switch)
  if (schema.type === 'boolean') {
    return (
      <div className="flex items-center justify-between rounded-lg border p-3 shadow-sm">
        <div className="space-y-0.5">
          <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
          {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
        </div>
        <Switch id={id} checked={value as boolean} onCheckedChange={onChange} className="scale-75" />
      </div>
    )
  }

  // 3. INTEGER / NUMBER
  if (schema.type === 'integer' || schema.type === 'number') {
    return (
      <div className="space-y-2">
        <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
        <Input
          id={id}
          type="number"
          className="h-8 text-xs"
          value={(value as number) ?? ''}
          min={schema.minimum as number | undefined}
          max={schema.maximum as number | undefined}
          onChange={(e) => {
            const val = e.target.value === '' ? undefined : Number(e.target.value)
            onChange(val)
          }}
        />
        {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
      </div>
    )
  }

  // 4. STRING (Default)
  if (schema.format === 'code') {
    return (
      <CodeEditorField
        id={id}
        label={label}
        description={description}
        value={(value as string) || ''}
        onChange={onChange}
        suggestions={suggestions}
        previewTheme={previewTheme}
        editorMeta={editorMeta}
      />
    )
  }

  return (
    <div className="space-y-2">
      <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
      <Input
        id={id}
        className="h-8 text-xs"
        value={(value as string) || ''}
        onChange={(e) => onChange(e.target.value)}
        placeholder={schema.default ? `Default: ${schema.default}` : ''}
      />
      {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
    </div>
  )
}

function CodeEditorField({
  id,
  label,
  description,
  value,
  onChange,
  suggestions = [],
  previewTheme,
  editorMeta,
}: {
  id: string
  label: string
  description?: string
  value: string
  onChange: (val: unknown) => void
  suggestions?: string[]
  previewTheme?: AutoFormProps['codePreviewTheme']
  editorMeta?: AutoFormProps['codeEditorMeta']
}) {
  const fileInputRef = React.useRef<HTMLInputElement | null>(null)
  const [isEditorOpen, setIsEditorOpen] = React.useState(false)
  const [draftValue, setDraftValue] = React.useState<string>(value || '')
  const [previewJson, setPreviewJson] = React.useState('')
  const { theme } = useTheme()
  const editorViewRef = React.useRef<unknown>(null)

  React.useEffect(() => {
    if (!isEditorOpen) {
      setDraftValue(value || '')
    }
  }, [value, isEditorOpen])

  const textValue = value || ''
  const trimmed = textValue.trim()
  const preview = trimmed ? trimmed.split('\n').slice(0, 3).join('\n') : 'No script configured'
  const templateKeys = suggestions.filter(Boolean)
  const hasSuggestions = templateKeys.length > 0
  const codeTheme = theme === 'dark' ? oneDark : undefined
  const [isTemplateMenuOpen, setIsTemplateMenuOpen] = React.useState(false)
  const firstTemplateKey = templateKeys[0] ?? 'login'
  const currentTemplateKey = editorMeta?.currentTemplateKey || firstTemplateKey
  const [previewMode, setPreviewMode] = React.useState<'render' | 'diff'>('render')
  const [validationState, setValidationState] = React.useState<
    'idle' | 'valid' | 'invalid' | 'empty'
  >('idle')

  const previewResult = parsePatchPayload(previewJson)
  const patchErrors: string[] = []
  if (previewResult.nodes) {
    validatePatchNodes(previewResult.nodes as PatchNode[], patchErrors, 'nodes')
  }
  const patchIssues = [...previewResult.errors, ...patchErrors]
  const insertSnippet = (snippet: string) => {
    const view = editorViewRef.current as {
      state?: { replaceSelection?: (text: string) => unknown }
      dispatch?: (tr: unknown) => void
      focus?: () => void
    } | null
    if (view?.state?.replaceSelection && view.dispatch) {
      const transaction = view.state.replaceSelection(snippet)
      view.dispatch(transaction)
      view.focus?.()
      return
    }
    setDraftValue((prev) => `${prev}${snippet}`)
  }

  const patchCompletion = (context: CompletionContext) => {
    const keyMatch = context.matchBefore(/"[^"]*$/)
    if (keyMatch) {
      let cursor = keyMatch.from - 1
      while (cursor >= 0 && /\s/.test(context.state.sliceDoc(cursor, cursor + 1))) {
        cursor -= 1
      }
      const prevChar = cursor >= 0 ? context.state.sliceDoc(cursor, cursor + 1) : ''
      if (prevChar === '{' || prevChar === ',') {
        return {
          from: keyMatch.from + 1,
          options: PATCH_PROPERTY_KEYS.map((value) => ({
            label: value,
            type: 'property',
          })),
          validFor: /[\w-]*/,
        }
      }
    }
    if (
      keyMatch &&
      context.state.sliceDoc(Math.max(0, keyMatch.from - 1), keyMatch.from) === '{'
    ) {
      return {
        from: keyMatch.from + 1,
        options: PATCH_PROPERTY_KEYS.map((value) => ({
          label: value,
          type: 'property',
        })),
        validFor: /[\w-]*/,
      }
    }

    const match = context.matchBefore(/"(type|component)"\s*:\s*"[^"]*$/)
    if (!match) return null
    const text = match.text
    const lastQuote = text.lastIndexOf('"')
    const from = match.from + Math.max(0, lastQuote + 1)
    const isType = text.includes('"type"')
    const options = (isType ? PATCH_NODE_TYPES : PATCH_COMPONENT_TYPES).map((value) => ({
      label: value,
      type: 'keyword',
    }))
    return {
      from,
      options,
      validFor: /[\w-]*/,
    }
  }

  const patchLinter = linter((view) => {
    const text = view.state.doc.toString()
    const parsed = parsePatchPayload(text)
    const errors: string[] = [...parsed.errors]
    if (parsed.nodes) {
      validatePatchNodes(parsed.nodes as PatchNode[], errors, 'nodes')
    }
    if (errors.length === 0) return []
    return [
      {
        from: 0,
        to: view.state.doc.length,
        severity: 'error',
        message: errors.join(' | '),
      } as Diagnostic,
    ]
  })

  const baseNodes = previewTheme?.nodes ?? []
  const baseJson = JSON.stringify(baseNodes, null, 2)
  const patchJson = previewResult.nodes ? JSON.stringify(previewResult.nodes, null, 2) : ''
  const diff = diffLines(baseJson.split('\n'), patchJson.split('\n'))
  const handleValidatePatch = () => {
    if (!previewJson.trim()) {
      setValidationState('empty')
      return
    }
    if (patchIssues.length > 0) {
      setValidationState('invalid')
    } else {
      setValidationState('valid')
    }
  }

  React.useEffect(() => {
    setValidationState('idle')
  }, [previewJson])

  return (
    <div className="space-y-2">
      <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">
        {label}
      </Label>
      <div className="bg-muted/30 text-muted-foreground min-h-[64px] whitespace-pre-wrap rounded-md border px-3 py-2 text-[10px] font-mono">
        {preview}
      </div>
      <div className="flex items-center justify-between">
        <div className="space-y-0.5">
          {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
          <p className="text-muted-foreground text-[10px]">
            Open the editor for the full script. Syntax highlighting is available there.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <input
            ref={fileInputRef}
            type="file"
            accept=".js,.mjs,.txt"
            className="hidden"
            onChange={(event) => {
              const file = event.target.files?.[0]
              if (!file) return
              const reader = new FileReader()
              reader.onload = () => {
                const content = typeof reader.result === 'string' ? reader.result : ''
                onChange(content)
                setDraftValue(content)
              }
              reader.readAsText(file)
            }}
          />
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="h-7 px-2 text-[10px]"
            onClick={() => fileInputRef.current?.click()}
          >
            Load file
          </Button>
          <Button
            type="button"
            size="sm"
            className="h-7 px-2 text-[10px]"
            onClick={() => setIsEditorOpen(true)}
          >
            Open editor
          </Button>
        </div>
      </div>
      <Dialog open={isEditorOpen} onOpenChange={setIsEditorOpen}>
        <DialogContent className="sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>{label}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <CodeMirror
              value={draftValue}
              height="320px"
              theme={codeTheme}
              extensions={[javascript()]}
              onChange={(val, viewUpdate) => {
                setDraftValue(val)
                editorViewRef.current = viewUpdate.view
              }}
              onCreateEditor={(view) => {
                editorViewRef.current = view
              }}
              basicSetup={{
                lineNumbers: true,
                highlightActiveLine: true,
                foldGutter: false,
              }}
            />
            {hasSuggestions && (
              <div className="flex flex-wrap items-center gap-2 text-[10px]">
                <span className="text-muted-foreground">Template keys:</span>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-[10px]"
                  onClick={() => insertSnippet(`\n  template_key: "${firstTemplateKey}"\n`)}
                >
                  Quick insert
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-[10px]"
                  onClick={() =>
                    insertSnippet(`\n  template_key: "${firstTemplateKey}"\n  ui_patch: []\n`)
                  }
                >
                  Insert + patch
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-[10px]"
                  onClick={() => insertSnippet(`\n  template_key: "${currentTemplateKey}"\n`)}
                >
                  Use current page
                </Button>
                <Popover open={isTemplateMenuOpen} onOpenChange={setIsTemplateMenuOpen}>
                  <PopoverTrigger asChild>
                    <Button variant="outline" size="sm" className="h-7 px-2 text-[10px]">
                      Search templates
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent align="start" className="w-64 p-0">
                    <Command>
                      <CommandInput placeholder="Search templates..." />
                      <CommandList>
                        <CommandEmpty>No templates found.</CommandEmpty>
                        <CommandGroup>
                          {templateKeys.map((key) => (
                            <CommandItem
                              key={key}
                              onSelect={() => {
                                insertSnippet(`\n  template_key: "${key}"\n`)
                                setIsTemplateMenuOpen(false)
                              }}
                            >
                              {key}
                            </CommandItem>
                          ))}
                        </CommandGroup>
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
              </div>
            )}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Label className="text-xs font-semibold">UI Patch Preview</Label>
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <button
                          type="button"
                          className="text-muted-foreground hover:text-foreground"
                        >
                          <CircleHelp className="h-3.5 w-3.5" />
                        </button>
                      </TooltipTrigger>
                      <TooltipContent side="right" className="max-w-xs">
                        <div className="space-y-1 text-[11px]">
                          <div className="font-semibold">Patch JSON schema</div>
                          <div>Root must be an array or {"{ \"nodes\": [...] }"}.</div>
                          <div>Each node needs a valid <code>type</code>.</div>
                          <div>Allowed types: {PATCH_NODE_TYPES.join(', ')}.</div>
                          <div>
                            Components must include <code>component</code>:
                            {` ${PATCH_COMPONENT_TYPES.join(', ')}`}.
                          </div>
                          <div>Nested nodes via <code>children</code> or <code>slots</code>.</div>
                        </div>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 px-2 text-[10px]"
                    onClick={handleValidatePatch}
                  >
                    Validate patch
                  </Button>
                  <Button
                    type="button"
                    variant={previewMode === 'render' ? 'default' : 'outline'}
                    size="sm"
                    className="h-7 px-2 text-[10px]"
                    onClick={() => setPreviewMode('render')}
                  >
                    Preview
                  </Button>
                  <Button
                    type="button"
                    variant={previewMode === 'diff' ? 'default' : 'outline'}
                    size="sm"
                    className="h-7 px-2 text-[10px]"
                    onClick={() => setPreviewMode('diff')}
                  >
                    Diff
                  </Button>
                </div>
              </div>
              <div className="overflow-hidden rounded-md border">
                <CodeMirror
                  value={previewJson}
                  height="140px"
                  theme={codeTheme}
                  extensions={[json(), autocompletion({ override: [patchCompletion] }), patchLinter]}
                  onChange={(val) => setPreviewJson(val)}
                  basicSetup={{
                    lineNumbers: true,
                    highlightActiveLine: true,
                    foldGutter: false,
                  }}
                />
              </div>
              {validationState === 'empty' && (
                <p className="text-xs text-amber-600">
                  Paste a patch JSON payload before validating.
                </p>
              )}
              {validationState === 'valid' && (
                <p className="text-xs text-emerald-600">Patch looks valid.</p>
              )}
              {validationState === 'invalid' && (
                <p className="text-xs text-red-600">Patch is invalid. Fix the issues below.</p>
              )}
              {patchIssues.length > 0 ? (
                <div className="space-y-1 text-xs text-red-600">
                  {patchIssues.map((issue) => (
                    <div key={issue}>{issue}</div>
                  ))}
                </div>
              ) : previewResult.nodes ? (
                <div className="h-64 overflow-hidden rounded-md border">
                  {previewMode === 'render' ? (
                    <FluidCanvas
                      tokens={
                        previewTheme?.tokens ?? {
                          colors: {
                            primary: '#2563eb',
                            background: '#f8fafc',
                            text: '#0f172a',
                            surface: '#ffffff',
                          },
                          typography: { font_family: 'system-ui', base_size: 14 },
                          radius: { base: 10 },
                          appearance: { mode: 'light' },
                        }
                      }
                      layout={previewTheme?.layout ?? { shell: 'CenteredCard', slots: ['main'] }}
                      blocks={previewResult.nodes}
                      assets={previewTheme?.assets ?? []}
                      selectedNodeId={null}
                      showChrome={false}
                      onSelectNode={() => {}}
                    />
                  ) : (
                    <div className="h-64 overflow-auto bg-slate-950 p-3 text-[10px] font-mono text-slate-200">
                      {baseNodes.length === 0 && (
                        <div className="mb-2 text-[10px] text-slate-400">
                          No base theme nodes available. Bind a theme to compare diffs.
                        </div>
                      )}
                      {diff.map((line, index) => (
                        <div
                          key={`${line.type}-${index}`}
                          className={
                            line.type === 'add'
                              ? 'bg-emerald-900/40 text-emerald-200'
                              : line.type === 'del'
                                ? 'bg-red-900/40 text-red-200'
                                : ''
                          }
                        >
                          <span className="mr-2 inline-block w-3">
                            {line.type === 'add' ? '+' : line.type === 'del' ? '-' : ' '}
                          </span>
                          {line.text}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ) : (
                <p className="text-muted-foreground text-[10px]">
                  Preview renders once valid JSON is provided.
                </p>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => {
                setDraftValue(textValue)
                setIsEditorOpen(false)
              }}
            >
              Cancel
            </Button>
            <Button
              type="button"
              size="sm"
              onClick={() => {
                onChange(draftValue)
                setIsEditorOpen(false)
              }}
            >
              Save Script
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
