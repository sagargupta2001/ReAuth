import { Plus } from 'lucide-react'

import { Button } from '@/shared/ui/button'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

export function GroupsPrimaryButtons() {
  const navigate = useRealmNavigate()
  return (
    <div className="flex gap-2">
      <Button className="space-x-1" onClick={() => navigate('/groups/new')}>
        <span>Create Group</span> <Plus size={18} />
      </Button>
    </div>
  )
}
