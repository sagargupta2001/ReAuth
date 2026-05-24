import { Plus } from 'lucide-react'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { Button } from '@/shared/ui/button'

export function IdentityProvidersPrimaryButtons() {
  const navigate = useRealmNavigate()

  return (
    <div className="flex gap-2">
      <Button className="space-x-1" onClick={() => navigate('/identity-providers/new')}>
        <span>Create</span> <Plus size={18} />
      </Button>
    </div>
  )
}
