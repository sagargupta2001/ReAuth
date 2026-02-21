import { useReactFlow } from '@xyflow/react'
import { ArrowLeft, CloudUpload, Loader2, Play, Save } from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useSaveDraft } from '@/features/flow-builder/api/useFlowDraft.ts'
import { usePublishFlow } from '@/features/flow-builder/api/usePublishFlow.ts'

interface BuilderHeaderProps {
  flowName: string
  flowId: string
  activeVersion?: number | null
}

export function BuilderHeader({ flowName, flowId, activeVersion }: BuilderHeaderProps) {
  const navigate = useRealmNavigate()
  const { toObject } = useReactFlow()

  const { mutateAsync: saveDraft, isPending: isSaving } = useSaveDraft()
  const { mutateAsync: publishFlow, isPending: isPublishing } = usePublishFlow()

  const isBusy = isSaving || isPublishing

  // ... handleSave and handlePublish logic (same as before) ...
  const handleSave = async () => {
    const graphData = toObject()
    await saveDraft({ draftId: flowId, graph: graphData })
    toast.success('Flow saved successfully')
  }

  const handlePublish = async () => {
    try {
      const graphData = toObject()
      await saveDraft({ draftId: flowId, graph: graphData })
      await publishFlow()
      // Optional: Refresh data here to update the activeVersion badge immediately
    } catch (error) {
      console.error(error)
    }
  }

  return (
    <header className="bg-muted/20 flex h-14 shrink-0 items-center justify-between border-b px-4">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Exit
        </Button>
        <Separator orientation="vertical" className="h-6" />

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold">{flowName}</span>

            {activeVersion ? (
              <Badge variant="outline" className="h-4 gap-1 px-1 text-[9px] font-normal">
                <div className="h-1.5 w-1.5 rounded-full bg-green-500" /> {/* Green Dot */}v
                {activeVersion}
              </Badge>
            ) : (
              <Badge
                variant="secondary"
                className="text-muted-foreground h-4 px-1 text-[9px] font-normal"
              >
                Unpublished
              </Badge>
            )}
          </div>

          <span className="text-muted-foreground text-[10px] tracking-wider uppercase">
            Flow Builder
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="outline" size="sm">
          <Play className="mr-2 h-3.5 w-3.5" /> Simulate
        </Button>

        <Button variant="secondary" size="sm" onClick={handleSave} disabled={isBusy}>
          {isSaving ? (
            <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
          ) : (
            <Save className="mr-2 h-3.5 w-3.5" />
          )}
          Save Draft
        </Button>

        <Button size="sm" onClick={handlePublish} disabled={isBusy} className="gap-2">
          {isPublishing ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <CloudUpload className="h-3.5 w-3.5" />
          )}
          Publish Flow
        </Button>
      </div>
    </header>
  )
}
