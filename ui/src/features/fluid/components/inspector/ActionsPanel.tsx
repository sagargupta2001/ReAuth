import { useState } from 'react'

import { X } from 'lucide-react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Label } from '@/shared/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import {
  actionKey,
  appendAction,
  patchAction,
  patchActionSignal,
  payloadEntriesOf,
  readRecentActionNodeIds,
  rememberActionNodeId,
  removeActionById,
  setPayloadMap,
  validatePayloadPath,
  writeRecentActionNodeIds,
  type InspectorAction,
  type PayloadEntry,
} from '@/features/fluid/lib/actionBindings'
import { cn } from '@/lib/utils'

const TRIGGERS = [
  { value: 'on_click', label: 'On Click' },
  { value: 'on_submit', label: 'On Submit' },
  { value: 'on_change', label: 'On Change' },
  { value: 'on_load', label: 'On Load' },
] as const

const SIGNAL_TYPES = [
  { value: 'submit_node', label: 'Submit Node' },
  { value: 'validate_node', label: 'Validate Node' },
  { value: 'call_subflow', label: 'Call Subflow' },
] as const

const RECENT_NODE_IDS_LIST = 'recent-action-node-ids'

interface ActionsPanelProps {
  actions: InspectorAction[]
  /** Input names on the page, for validating and suggesting `inputs.*` paths. */
  inputNames: string[]
  /** Context paths offered alongside the inputs. */
  contextSuggestions: string[]
  onChange: (next: InspectorAction[]) => void
}

/**
 * Controlled editor for a block's action bindings.
 *
 * All mutation goes through the pure transforms in `lib/actionBindings.ts`, so
 * the fiddly semantics — dropping blank payload keys, removing `payload_map`
 * when it empties — are unit-tested rather than buried in JSX.
 */
export function ActionsPanel({
  actions,
  inputNames,
  contextSuggestions,
  onChange,
}: ActionsPanelProps) {
  const [recentNodeIds, setRecentNodeIds] = useState<string[]>(() =>
    readRecentActionNodeIds(),
  )

  const rememberNodeId = (value: string) => {
    setRecentNodeIds((prev) => {
      const next = rememberActionNodeId(value, prev)
      writeRecentActionNodeIds(next)
      return next
    })
  }

  return (
    <div className="space-y-4">
      <div className="text-muted-foreground text-[11px]">
        Actions emit signals from this block into the auth flow.
      </div>

      {actions.length === 0 ? (
        <div className="text-muted-foreground text-xs">No actions configured yet.</div>
      ) : (
        actions.map((action, index) => {
          const id = actionKey(action, index)
          const signal = action.signal ?? {}
          const entries = payloadEntriesOf(action)

          const writeEntries = (next: PayloadEntry[]) =>
            onChange(setPayloadMap(actions, id, next))

          return (
            <div key={id} className="space-y-3 rounded-md border p-3">
              <div className="flex items-center justify-between">
                <div className="text-xs font-semibold">Action {index + 1}</div>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={`Remove action ${index + 1}`}
                  className="h-7 w-7"
                  onClick={() => onChange(removeActionById(actions, id))}
                >
                  <X className="h-3.5 w-3.5" />
                </Button>
              </div>

              <div className="space-y-2">
                <Label className="text-xs">Trigger</Label>
                <Select
                  value={String(action.trigger || 'on_click')}
                  onValueChange={(value) => onChange(patchAction(actions, id, { trigger: value }))}
                >
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="Select trigger" />
                  </SelectTrigger>
                  <SelectContent>
                    {TRIGGERS.map((trigger) => (
                      <SelectItem key={trigger.value} value={trigger.value}>
                        {trigger.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label className="text-xs">Signal Type</Label>
                <Select
                  value={String(signal.type || 'submit_node')}
                  onValueChange={(value) =>
                    onChange(patchActionSignal(actions, id, { type: value }))
                  }
                >
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="Select signal type" />
                  </SelectTrigger>
                  <SelectContent>
                    {SIGNAL_TYPES.map((type) => (
                      <SelectItem key={type.value} value={type.value}>
                        {type.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label className="text-xs">Node ID (optional)</Label>
                <Input
                  className="h-8 text-xs"
                  value={String(signal.node_id || '')}
                  placeholder="e.g. auth-password"
                  list={RECENT_NODE_IDS_LIST}
                  onChange={(event) =>
                    onChange(patchActionSignal(actions, id, { node_id: event.target.value }))
                  }
                  onBlur={(event) => rememberNodeId(event.target.value)}
                />
                {recentNodeIds.length > 0 && (
                  <datalist id={RECENT_NODE_IDS_LIST}>
                    {recentNodeIds.map((nodeId) => (
                      <option key={`recent-node-${nodeId}`} value={nodeId} />
                    ))}
                  </datalist>
                )}
                <p className="text-muted-foreground text-[10px]">
                  Autocomplete uses recently entered node IDs. Freeform values are allowed.
                </p>
              </div>

              <div className="space-y-2">
                <Label className="text-xs">Payload Map</Label>
                <div className="space-y-2">
                  {entries.length === 0 ? (
                    <div className="text-muted-foreground text-[11px]">
                      No payload mapping yet.
                    </div>
                  ) : (
                    entries.map((entry, entryIndex) => {
                      const validation = validatePayloadPath(entry.path, inputNames)
                      const listId = `payload-map-${index}-${entryIndex}`
                      return (
                        <div key={`${entry.key}-${entryIndex}`} className="space-y-1">
                          <div className="flex items-center gap-2">
                            <Input
                              className="h-8 text-xs"
                              placeholder="payload key"
                              aria-label="Payload key"
                              value={entry.key}
                              onChange={(event) => {
                                const next = [...entries]
                                next[entryIndex] = { ...entry, key: event.target.value }
                                writeEntries(next)
                              }}
                            />
                            <Input
                              list={listId}
                              aria-label="Payload path"
                              aria-invalid={!validation.valid}
                              className={cn(
                                'h-8 text-xs',
                                !validation.valid &&
                                  'border-destructive focus-visible:ring-destructive',
                              )}
                              placeholder="inputs.email"
                              value={entry.path}
                              onChange={(event) => {
                                const next = [...entries]
                                next[entryIndex] = { ...entry, path: event.target.value }
                                writeEntries(next)
                              }}
                            />
                            <Button
                              variant="ghost"
                              size="icon"
                              aria-label="Remove mapping"
                              className="h-8 w-8"
                              onClick={() =>
                                writeEntries(entries.filter((_, i) => i !== entryIndex))
                              }
                            >
                              <X className="h-3.5 w-3.5" />
                            </Button>
                          </div>
                          <datalist id={listId}>
                            {inputNames.map((name) => (
                              <option key={`input-${name}`} value={`inputs.${name}`} />
                            ))}
                            {contextSuggestions.map((path) => (
                              <option key={`context-${path}`} value={path} />
                            ))}
                          </datalist>
                          {!validation.valid && (
                            <div className="text-destructive text-[10px]">
                              {validation.message}
                            </div>
                          )}
                        </div>
                      )
                    })
                  )}
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-8 text-xs"
                    onClick={() =>
                      writeEntries([
                        ...entries,
                        { key: `payload_${entries.length + 1}`, path: '' },
                      ])
                    }
                  >
                    Add mapping
                  </Button>
                  <p className="text-muted-foreground text-[10px]">
                    Use paths like <code>inputs.email</code> or <code>context.client_id</code>.
                  </p>
                </div>
              </div>
            </div>
          )
        })
      )}

      <Button
        variant="outline"
        size="sm"
        className="h-8 text-xs"
        onClick={() => onChange(appendAction(actions))}
      >
        Add action
      </Button>
    </div>
  )
}
