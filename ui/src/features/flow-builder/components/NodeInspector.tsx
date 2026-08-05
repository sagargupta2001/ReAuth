import { Check, ChevronDown, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'

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
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
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
  const supportsUi = nodeDefinition?.capabilities?.supports_ui ?? false

  const currentConfig = (selectedNode?.data?.config as Record<string, unknown>) || {}
  const currentUi =
    typeof currentConfig.ui === 'object' && currentConfig.ui
      ? (currentConfig.ui as Record<string, unknown>)
      : {}
  const fallbackTemplate = nodeDefinition?.default_template_key ?? undefined
  const explicitTemplate =
    typeof currentUi.page_key === 'string'
      ? currentUi.page_key
      : typeof currentConfig.template_key === 'string'
        ? currentConfig.template_key
        : undefined
  const currentTemplate = explicitTemplate ?? fallbackTemplate

  const availablePages = useMemo(() => activeTheme?.pages ?? [], [activeTheme?.pages])
  const allowedCategories = useMemo(
    () => nodeDefinition?.capabilities?.allowed_page_categories ?? [],
    [nodeDefinition?.capabilities?.allowed_page_categories],
  )
  const filteredPages = useMemo(() => {
    if (!allowedCategories.length) return availablePages
    return availablePages.filter(
      (page) =>
        allowedCategories.includes(page.category) || page.category === 'custom',
    )
  }, [availablePages, allowedCategories])
  const [isTemplateOpen, setIsTemplateOpen] = useState(false)
  const selectedPage = useMemo(
    () => availablePages.find((page) => page.key === currentTemplate),
    [availablePages, currentTemplate],
  )
  const templateAllowed = useMemo(() => {
    if (!currentTemplate) return true
    if (!selectedPage) return true
    if (!allowedCategories.length) return true
    if (selectedPage.category === 'custom') return true
    return allowedCategories.includes(selectedPage.category)
  }, [allowedCategories, currentTemplate, selectedPage])
  const templateExists = !activeTheme
    ? true
    : currentTemplate
      ? availablePages.some((page) => page.key === currentTemplate)
      : true

  const [activeTab, setActiveTab] = useState('general')

  // Selecting a different node resets the panel to the first tab, so a tab the
  // new node does not render (Template is conditional on supports_ui) can never
  // stay active and leave the panel blank.
  useEffect(() => {
    setActiveTab('general')
  }, [selectedNodeId])

  // Guards the render between the selection change and the effect above
  // committing, which would otherwise flash an empty panel.
  const currentTab = activeTab === 'template' && !supportsUi ? 'general' : activeTab

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
    const ui = currentConfig.ui
    const nextConfig = { ...newConfig } as Record<string, unknown>
    if (templateKey && !('template_key' in nextConfig)) {
      nextConfig.template_key = templateKey
    }
    if (ui && !('ui' in nextConfig)) {
      nextConfig.ui = ui
    }
    updateNodeData(selectedNode.id, {
      ...selectedNode.data,
      config: nextConfig,
    })
  }

  const handleTemplateChange = (value?: string) => {
    const nextConfig = { ...currentConfig }
    const nextUi = { ...(currentUi || {}) }
    if (!value) {
      delete nextUi.page_key
      delete nextConfig.template_key
    } else {
      nextUi.page_key = value
      delete nextConfig.template_key
    }
    if (Object.keys(nextUi).length) {
      nextConfig.ui = nextUi
    } else {
      delete nextConfig.ui
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
      <div className="flex h-14 shrink-0 items-center justify-between px-4">
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
      <Tabs
        value={currentTab}
        onValueChange={setActiveTab}
        className="flex min-h-0 flex-1 flex-col gap-0"
      >
        <div className="shrink-0 px-4">
          <TabsList variant="line">
            <TabsTrigger variant="line" value="general">
              General
            </TabsTrigger>
            {supportsUi && (
              <TabsTrigger variant="line" value="template">
                Template
              </TabsTrigger>
            )}
            <TabsTrigger variant="line" value="parameters">
              Parameters
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="custom-scrollbar min-h-0 flex-1 overflow-y-auto p-4">
          {/* Tab: General */}
          <TabsContent value="general" className="mt-0 space-y-3">
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
          </TabsContent>

          {/* Tab: Template */}
          {supportsUi && (
            <TabsContent value="template" className="mt-0 space-y-3">
              {nodeDefinition?.capabilities?.ui_surface ? (
                <div className="text-muted-foreground text-[10px] uppercase tracking-wide">
                  UI Surface: {nodeDefinition.capabilities.ui_surface.replace('_', ' ')}
                </div>
              ) : null}
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
                            {!explicitTemplate && <Check className="h-3.5 w-3.5 text-primary" />}
                          </CommandItem>
                        </CommandGroup>
                        <CommandGroup>
                          {filteredPages.map((page) => (
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
                  {allowedCategories.length
                    ? ` Allowed categories: ${allowedCategories.join(', ')}.`
                    : ''}
                </p>
              </div>
              {!templateExists && currentTemplate && (
                <Alert variant="destructive">
                  <AlertTitle>Missing template</AlertTitle>
                  <AlertDescription>
                    The active theme does not define the page “{currentTemplate}”. Users will fall
                    back to the system template.
                  </AlertDescription>
                </Alert>
              )}
              {templateExists && !templateAllowed && selectedPage && (
                <Alert>
                  <AlertTitle>Template category mismatch</AlertTitle>
                  <AlertDescription>
                    This node expects pages in: {allowedCategories.join(', ')}. The selected page
                    is categorized as {selectedPage.category}.
                  </AlertDescription>
                </Alert>
              )}
            </TabsContent>
          )}

          {/* Tab: Parameters */}
          <TabsContent value="parameters" className="mt-0">
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
          </TabsContent>
        </div>
      </Tabs>

      {/* Footer / Debug Info (Optional) */}
      <div className="bg-muted/20 border-t p-2">
        <p className="text-muted-foreground/50 text-center text-[10px]">
          {nodeDefinition?.description || 'Standard Node'}
        </p>
      </div>
    </aside>
  )
}
