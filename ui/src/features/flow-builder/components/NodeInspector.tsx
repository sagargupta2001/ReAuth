import { Check, ChevronDown, X } from 'lucide-react'
import { useMemo, useState } from 'react'

import { Button } from '@/components/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/command'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import { Separator } from '@/components/separator'
import { AutoForm } from '@/shared/ui/auto-form'
import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'
import { useActiveTheme } from '@/features/theme/api/useActiveTheme'

import { useFlowBuilderStore } from '../store/flowBuilderStore'

export function NodeInspector() {
  const selectedNodeId = useFlowBuilderStore((s) => s.selectedNodeId)
  const nodes = useFlowBuilderStore((s) => s.nodes)
  const nodeTypes = useFlowBuilderStore((s) => s.nodeTypes) // Need to ensure this is exposed in store
  const selectNode = useFlowBuilderStore((s) => s.selectNode)
  const updateNodeData = useFlowBuilderStore((s) => s.updateNodeData)
  const { data: activeTheme } = useActiveTheme()

  const selectedNode = nodes.find((n) => n.id === selectedNodeId)
  const nodeType = selectedNode?.type ?? ''

  // 1. Lookup Schema based on Node Type (e.g., "core.auth.password")
  const nodeDefinition = nodeType ? nodeTypes.find((t) => t.id === nodeType) : undefined
  const configSchema = nodeDefinition?.config_schema
  const supportsUi = nodeDefinition?.supports_ui ?? false

  const currentConfig = (selectedNode?.data?.config as Record<string, unknown>) || {}
  const fallbackTemplate = nodeDefinition?.default_template_key ?? undefined
  const explicitTemplate =
    typeof currentConfig.template_key === 'string' ? currentConfig.template_key : undefined
  const currentTemplate = explicitTemplate ?? fallbackTemplate

  const availablePages = useMemo(() => activeTheme?.pages ?? [], [activeTheme?.pages])
  const [isTemplateOpen, setIsTemplateOpen] = useState(false)
  const selectedPage = useMemo(
    () => availablePages.find((page) => page.key === currentTemplate),
    [availablePages, currentTemplate],
  )
  const templateExists = !activeTheme
    ? true
    : currentTemplate
      ? availablePages.some((page) => page.key === currentTemplate)
      : true

  if (!selectedNode) return null

  // 2. Handlers
  const handleLabelChange = (label: string) => {
    updateNodeData(selectedNode.id, {
      ...selectedNode.data,
      label,
    })
  }

  const handleConfigChange = (newConfig: Record<string, unknown>) => {
    const templateKey = currentConfig.template_key
    updateNodeData(selectedNode.id, {
      ...selectedNode.data,
      config: templateKey && !('template_key' in newConfig)
        ? { ...newConfig, template_key: templateKey }
        : newConfig,
    })
  }

  const handleTemplateChange = (value?: string) => {
    const nextConfig = { ...currentConfig }
    if (!value) {
      delete nextConfig.template_key
    } else {
      nextConfig.template_key = value
    }
    updateNodeData(selectedNode.id, {
      ...selectedNode.data,
      config: {
        ...nextConfig,
      },
    })
  }

  return (
    <aside className="bg-background z-20 flex h-full w-80 shrink-0 flex-col border-l shadow-xl transition-all duration-300 ease-in-out">
      {/* Header */}
      <div className="flex h-14 shrink-0 items-center justify-between border-b px-4">
        <div className="flex flex-col">
          <h3 className="text-sm font-semibold">Configuration</h3>
          <span className="text-muted-foreground font-mono text-[10px] tracking-wider uppercase">
            {nodeType || 'unknown'}
          </span>
        </div>
        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => selectNode(null)}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      {/* Content */}
      <div className="custom-scrollbar flex-1 space-y-6 overflow-y-auto p-4">
        {/* Section 1: General Info */}
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <div className="h-1.5 w-1.5 rounded-full bg-blue-500" />
            <h4 className="text-muted-foreground text-xs font-bold tracking-wider uppercase">
              General
            </h4>
          </div>

          <div className="border-muted ml-0.5 space-y-3 border-l-2 pl-3.5">
            <div className="space-y-1.5">
              <Label className="text-xs font-medium">Node Label</Label>
              <Input
                className="bg-muted/30 h-8 text-xs"
                value={selectedNode.data.label as string}
                onChange={(e) => handleLabelChange(e.target.value)}
              />
            </div>

            <div className="space-y-1.5">
              <Label className="text-xs font-medium">Internal ID</Label>
              <div className="bg-muted text-muted-foreground rounded-md border px-3 py-1.5 font-mono text-[10px] break-all">
                {selectedNode.id}
              </div>
            </div>
          </div>
        </div>

        <Separator />

        {supportsUi && (
          <>
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <div className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                <h4 className="text-muted-foreground text-xs font-bold tracking-wider uppercase">
                  Template
                </h4>
              </div>

              <div className="border-muted ml-0.5 space-y-3 border-l-2 pl-3.5">
                <div className="space-y-1.5">
                  <Label className="text-xs font-medium">Page Template</Label>
                  <Popover open={isTemplateOpen} onOpenChange={setIsTemplateOpen}>
                    <PopoverTrigger asChild>
                      <Button variant="outline" size="sm" className="h-8 w-full justify-between">
                        <span className="text-xs font-semibold">
                          {selectedPage?.label || currentTemplate || 'Select template'}
                        </span>
                        <ChevronDown className="text-muted-foreground h-3.5 w-3.5" />
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent align="start" className="w-72 p-0">
                      <Command>
                        <CommandInput placeholder="Search templates..." />
                        <CommandList>
                          <CommandEmpty>No templates found.</CommandEmpty>
                          <CommandGroup>
                            <CommandItem
                              onSelect={() => {
                                handleTemplateChange(undefined)
                                setIsTemplateOpen(false)
                              }}
                            >
                              <span className="flex flex-1 flex-col">
                                <span className="text-xs font-medium">Use default</span>
                                <span className="text-muted-foreground text-[10px]">
                                  {fallbackTemplate
                                    ? `Default template: ${fallbackTemplate}`
                                    : 'Clear explicit binding'}
                                </span>
                              </span>
                              {!explicitTemplate && (
                                <Check className="h-3.5 w-3.5 text-primary" />
                              )}
                            </CommandItem>
                          </CommandGroup>
                          <CommandGroup>
                            {availablePages.map((page) => (
                              <CommandItem
                                key={page.key}
                                onSelect={() => {
                                  handleTemplateChange(page.key)
                                  setIsTemplateOpen(false)
                                }}
                              >
                                <span className="flex flex-1 flex-col">
                                  <span className="text-xs font-medium">{page.label}</span>
                                  <span className="text-muted-foreground text-[10px]">
                                    {page.description}
                                  </span>
                                </span>
                                {page.key === currentTemplate && (
                                  <Check className="h-3.5 w-3.5 text-primary" />
                                )}
                              </CommandItem>
                            ))}
                          </CommandGroup>
                        </CommandList>
                      </Command>
                    </PopoverContent>
                  </Popover>
                  <Input
                    className="bg-muted/30 h-8 text-xs"
                    value={explicitTemplate || ''}
                    onChange={(event) => handleTemplateChange(event.target.value)}
                    placeholder="Custom template key"
                  />
                  <p className="text-muted-foreground text-[10px]">
                    Assign a Fluid page key to this node.
                  </p>
                </div>
                {!templateExists && currentTemplate && (
                  <Alert variant="destructive">
                    <AlertTitle>Missing template</AlertTitle>
                    <AlertDescription>
                      The active theme does not define the page “{currentTemplate}”. Users will
                      fall back to the system template.
                    </AlertDescription>
                  </Alert>
                )}
              </div>
            </div>

            <Separator />
          </>
        )}

        {/* Section 2: Dynamic Parameters */}
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <div className="h-1.5 w-1.5 rounded-full bg-purple-500" />
            <h4 className="text-muted-foreground text-xs font-bold tracking-wider uppercase">
              Parameters
            </h4>
          </div>

          <div className="pl-1">
            {configSchema && Object.keys(configSchema.properties || {}).length > 0 ? (
              <AutoForm
                schema={configSchema}
                values={(selectedNode.data.config as Record<string, unknown>) || {}}
                onChange={handleConfigChange}
              />
            ) : (
              <div className="rounded-lg border border-dashed p-4 text-center">
                <p className="text-muted-foreground text-xs italic">
                  No configurable parameters for this node.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Footer / Debug Info (Optional) */}
      <div className="bg-muted/20 border-t p-2">
        <p className="text-muted-foreground/50 text-center text-[10px]">
          {nodeDefinition?.description || 'Standard Node'}
        </p>
      </div>
    </aside>
  )
}
