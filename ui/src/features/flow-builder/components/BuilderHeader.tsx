import { ArrowLeft, Play, Save } from 'lucide-react'

import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'

// Fix: Add flowId to the interface
interface BuilderHeaderProps {
  flowName: string
  flowId: string
}

export function BuilderHeader({ flowName, flowId }: BuilderHeaderProps) {
  const navigate = useRealmNavigate()

  return (
    <header className="bg-muted/20 flex h-14 shrink-0 items-center justify-between border-b px-4">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Exit
        </Button>
        <Separator orientation="vertical" className="h-6" />
        <div className="flex flex-col">
          <span className="text-sm font-semibold">{flowName}</span>
          <span className="text-muted-foreground text-[10px] tracking-wider uppercase">
            Flow Builder
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="outline" size="sm">
          <Play className="mr-2 h-3.5 w-3.5" /> Simulate
        </Button>
        <Button size="sm" onClick={() => console.log('Saving flow', flowId)}>
          <Save className="mr-2 h-3.5 w-3.5" /> Save Flow
        </Button>
      </div>
    </header>
  )
}
