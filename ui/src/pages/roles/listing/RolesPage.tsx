import { RolesPrimaryButtons } from '@/features/roles/components/RolesPrimaryButtons.tsx'
import { RolesTable } from '@/features/roles/components/RolesTable.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function RolesPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Roles</h2>
          <p className="text-muted-foreground">Define authorities and assign permissions.</p>
        </div>
        <RolesPrimaryButtons />
      </div>
      <RolesTable />
    </Main>
  )
}
