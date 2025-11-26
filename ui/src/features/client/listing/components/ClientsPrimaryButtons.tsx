import { Plus } from 'lucide-react'

import { Button } from '@/components/button'

export function ClientsPrimaryButtons() {
  return (
    <div className="flex gap-2">
      <Button className="space-x-1">
        <span>Create</span> <Plus size={18} />
      </Button>
    </div>
  )
}
