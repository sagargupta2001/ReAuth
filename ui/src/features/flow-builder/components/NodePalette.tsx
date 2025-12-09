import { Lock, ShieldAlert, Split } from 'lucide-react'

import { cn } from '@/lib/utils'

// todo We will fetch these from the API later (Milestone 1 /nodes endpoint)
const AVAILABLE_NODES = [
  { type: 'authenticator.password', label: 'Password Form', icon: Lock, category: 'Auth' },
  { type: 'authenticator.otp', label: 'OTP Input', icon: ShieldAlert, category: 'Auth' },
  { type: 'logic.condition', label: 'Condition', icon: Split, category: 'Logic' },
]

export function NodePalette() {
  const onDragStart = (event: React.DragEvent, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType)
    event.dataTransfer.effectAllowed = 'move'
  }

  return (
    <aside className="bg-muted/10 flex w-64 flex-col border-r">
      <div className="text-muted-foreground border-b p-4 text-xs font-semibold uppercase">
        Components
      </div>
      <div className="flex-1 space-y-4 overflow-y-auto p-4">
        {AVAILABLE_NODES.map((node) => (
          <div
            key={node.type}
            className={cn(
              'bg-card hover:border-primary/50 flex cursor-grab items-center gap-3 rounded-md border p-3 shadow-sm transition-colors active:cursor-grabbing',
            )}
            draggable
            onDragStart={(e) => onDragStart(e, node.type)}
          >
            <node.icon className="text-muted-foreground h-4 w-4" />
            <span className="text-sm font-medium">{node.label}</span>
          </div>
        ))}
      </div>
    </aside>
  )
}
