import { useEffect } from 'react'

import { ArrowLeft, Settings, ShieldCheck } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { buttonVariants } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { UserRolesTab } from '@/features/user/components/UserRolesTab'
import { EditUserForm } from '@/features/user/forms/EditUserForm.tsx'
import { cn } from '@/lib/utils'

export function EditUserPage() {
  const { userId, tab } = useParams<{ userId: string; tab?: string }>()
  const navigate = useRealmNavigate()

  if (!userId) return null

  const validTabs = ['settings', 'roles']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'settings'

  useEffect(() => {
    if (!tab) {
      navigate(`/users/${userId}/settings`, { replace: true })
    }
  }, [tab, userId, navigate])

  const handleTabChange = (newTab: string) => navigate(`/users/${userId}/${newTab}`)

  return (
    <div className="bg-background flex h-full w-full flex-col overflow-hidden p-6">
      <div className="mb-2 shrink-0">
        <RealmLink
          to="/users"
          className={cn(
            buttonVariants({ variant: 'link', size: 'sm' }),
            'text-muted-foreground hover:text-foreground gap-2 pl-0',
          )}
        >
          <ArrowLeft className="h-4 w-4" /> Back to Users
        </RealmLink>
      </div>

      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 shrink-0 border-b px-6 pt-2">
          <TabsList className="gap-6 bg-transparent p-0">
            <TabsTrigger value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>
            <TabsTrigger value="roles" className="tab-trigger-styles">
              <ShieldCheck className="mr-2 h-4 w-4" /> Roles
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 flex-1 overflow-y-auto">
          <TabsContent value="settings" className="mt-0 h-full w-full p-6">
            <EditUserForm userId={userId} />
          </TabsContent>
          <TabsContent value="roles" className="mt-0 h-full w-full p-6">
            <UserRolesTab userId={userId} />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
