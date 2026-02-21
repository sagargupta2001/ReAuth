import { Plus } from 'lucide-react'

import { Button } from '@/shared/ui/button.tsx'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

export function ClientsPrimaryButtons() {
  const navigate = useRealmNavigate()
  return (
    <div className="flex gap-2">
      <Button className="space-x-1" onClick={() => navigate('/clients/new')}>
        <span>Create</span> <Plus size={18} />
      </Button>
    </div>
  )
}
