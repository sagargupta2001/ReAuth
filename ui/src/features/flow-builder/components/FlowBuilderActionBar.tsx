import { useReactFlow } from '@xyflow/react'
import { toast } from 'sonner'

import { useSaveDraft } from '@/features/flow-builder/api/useFlowDraft'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore'
import { BuilderFloatingActionBar } from '@/components/builder-floating-action-bar'

interface FlowBuilderActionBarProps {
  flowId: string
}

export function FlowBuilderActionBar({ flowId }: FlowBuilderActionBarProps) {
  const { toObject } = useReactFlow()
  const { undo, redo, past, future } = useFlowBuilderStore()
  const { mutateAsync: saveDraft, isPending: isSaving } = useSaveDraft()

  const handleSave = async () => {
    const graphData = toObject()
    await saveDraft({ draftId: flowId, graph: graphData })
    toast.success('Flow saved successfully')
  }

  return (
    <BuilderFloatingActionBar
      canUndo={past.length > 0}
      canRedo={future.length > 0}
      onUndo={undo}
      onRedo={redo}
      onSave={() => void handleSave()}
      isSaving={isSaving}
    />
  )
}
