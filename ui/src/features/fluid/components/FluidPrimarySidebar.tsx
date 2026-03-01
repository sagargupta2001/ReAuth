import { Layers, Settings2 } from 'lucide-react'

import { Button } from '@/components/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/tooltip'
import { cn } from '@/lib/utils'

type FluidPrimaryPanel = 'sections' | 'settings'

interface FluidPrimarySidebarProps {
  activePanel: FluidPrimaryPanel
  onSelectPanel: (panel: FluidPrimaryPanel) => void
}

export function FluidPrimarySidebar({
  activePanel,
  onSelectPanel,
}: FluidPrimarySidebarProps) {
  return (
    <TooltipProvider>
      <aside className="bg-muted/10 flex w-14 flex-col items-center border-r py-4">
        <div className="flex flex-col gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={activePanel === 'sections' ? 'secondary' : 'ghost'}
                size="icon"
                className={cn('h-9 w-9', activePanel === 'sections' && 'shadow-sm')}
                onClick={() => onSelectPanel('sections')}
              >
                <Layers className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">Sections</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={activePanel === 'settings' ? 'secondary' : 'ghost'}
                size="icon"
                className={cn('h-9 w-9', activePanel === 'settings' && 'shadow-sm')}
                onClick={() => onSelectPanel('settings')}
              >
                <Settings2 className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">Theme Settings</TooltipContent>
          </Tooltip>
        </div>
      </aside>
    </TooltipProvider>
  )
}
