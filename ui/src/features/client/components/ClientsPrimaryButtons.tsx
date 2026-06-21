import { CreateClientDialog } from '@/features/client/components/CreateClientDialog.tsx'

export function ClientsPrimaryButtons() {
  return (
    <div className="flex gap-2">
      <CreateClientDialog />
    </div>
  )
}
