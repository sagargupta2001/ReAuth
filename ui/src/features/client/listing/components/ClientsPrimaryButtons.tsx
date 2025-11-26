import { Plus } from 'lucide-react'

import { Button } from '@/components/button'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'

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
