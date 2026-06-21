import { Plus } from 'lucide-react'

import { Button } from '@/shared/ui/button.tsx'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

export function ClientsPrimaryButtons() {
  const navigate = useRealmNavigate()
  return (
    <div className="flex gap-2">
      <Button size="sm" className="flex items-center gap-2" onClick={() => navigate('/clients/new')}>
        <Plus size={18} />
        <span>Create Client</span>
      </Button>
    </div>
  )
}
