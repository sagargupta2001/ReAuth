import { useEffect } from 'react'

import { ArrowLeft, KeyRound, Settings, ShieldCheck, UserRoundPen } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { buttonVariants } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { UserCredentialsTab } from '@/features/user/components/UserCredentialsTab'
import { UserRolesTab } from '@/features/user/components/UserRolesTab'
import { EditUserForm } from '@/features/user/forms/EditUserForm.tsx'
import { cn } from '@/lib/utils'

export function EditUserPage() {
  const { userId, tab } = useParams<{ userId: string; tab?: string }>()
  const navigate = useRealmNavigate()

  const validTabs = ['profile', 'settings', 'roles', 'credentials']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'profile'

  const handleTabChange = (newTab: string) => userId && navigate(`/users/${userId}/${newTab}`)

  useEffect(() => {
    if (!tab && userId) navigate(`/users/${userId}/profile`, { replace: true })
  }, [tab, userId, navigate])

  if (!userId) return null

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
          <ArrowLeft className="h-4 w-4" />
          Back to Users
        </RealmLink>
      </div>

      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 shrink-0 border-b px-6 pt-2">
          <TabsList variant="line" className="gap-6 bg-transparent p-0">
            <TabsTrigger variant="line" value="profile" className="tab-trigger-styles">
              <UserRoundPen className="mr-2 h-4 w-4" /> Profile
            </TabsTrigger>
            <TabsTrigger variant="line" value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>
            <TabsTrigger variant="line" value="roles" className="tab-trigger-styles">
              <ShieldCheck className="mr-2 h-4 w-4" /> Roles
            </TabsTrigger>
            <TabsTrigger variant="line" value="credentials" className="tab-trigger-styles">
              <KeyRound className="mr-2 h-4 w-4" /> Credentials
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 flex-1 overflow-y-auto">
          <TabsContent value="profile" className="mt-0 h-full w-full p-6">
            <EditUserForm userId={userId} />
          </TabsContent>
          <TabsContent value="settings" className="mt-0 h-full w-full p-6">
            <EditUserForm userId={userId} />
          </TabsContent>
          <TabsContent value="roles" className="mt-0 h-full w-full p-6">
            <UserRolesTab userId={userId} />
          </TabsContent>
          <TabsContent value="credentials" className="mt-0 h-full w-full p-6">
            <UserCredentialsTab userId={userId} />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
