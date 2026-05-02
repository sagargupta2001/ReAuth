import { Plus } from 'lucide-react'

import { Button } from '@/shared/ui/button.tsx'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

export function UsersPrimaryButtons() {
  const navigate = useRealmNavigate()
  return (
    <div className="flex gap-2">
      <Button size="sm" onClick={() => navigate('/users/new')} className="flex items-center gap-2">
        <Plus size={18} />
        <span>Create User</span>
      </Button>
    </div>
  )
}
