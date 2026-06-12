import { RolesTable } from '@/features/roles/components/RolesTable.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function RolesPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6 p-12">
      <h2 className="text-2xl font-bold tracking-tight">Roles</h2>
      <RolesTable />
    </Main>
  )
}
