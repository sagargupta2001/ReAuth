import { useEffect } from 'react'

import { format } from 'date-fns'
import { ArrowLeft, KeyRound, Settings, ShieldCheck, UserRound, UserRoundPen } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { buttonVariants } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useUser } from '@/features/user/api/useUser.ts'
import { UserCredentialsTab } from '@/features/user/components/UserCredentialsTab'
import { UserRolesTab } from '@/features/user/components/UserRolesTab'
import { UseProfileTab } from '@/features/user/components/UseProfileTab.tsx'
import { cn } from '@/lib/utils'
import { Skeleton } from '@/shared/ui/skeleton.tsx'

export function EditUserPage() {
  const { userId, tab } = useParams<{ userId: string; tab?: string }>()
  const navigate = useRealmNavigate()

  const { data: user, isLoading: isUserLoading } = useUser(userId as string)

  const validTabs = ['profile', 'settings', 'roles', 'credentials']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'profile'

  const handleTabChange = (newTab: string) => userId && navigate(`/users/${userId}/${newTab}`)

  useEffect(() => {
    if (!tab && userId) navigate(`/users/${userId}/profile`, { replace: true })
  }, [tab, userId, navigate])

  if (!userId) return null

  const userIconSkeleton = () => {
    return (
      <div className="flex items-center gap-4">
        <div className="border-primary/10 flex items-center justify-center rounded-full border p-4">
          <Skeleton className="h-7 w-7 rounded-full" />
        </div>
        <div className="grid gap-2">
          <Skeleton className="h-7 w-32" />
          <Skeleton className="h-4 w-48 opacity-70" />
        </div>
      </div>
    )
  }

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

      {isUserLoading ? (
        userIconSkeleton()
      ) : (
        <div className="flex items-center gap-4">
          <div className="border-primary flex items-center justify-center rounded-full border p-4">
            <UserRound className="h-7 w-7" />
          </div>

          <div className="grid">
            <span className="text-2xl font-semibold">{user?.username}</span>
            {user?.last_sign_in_at && (
              <span className="text-muted-foreground text-sm">
                Last active{' '}
                {format(new Date(user?.last_sign_in_at as string), 'MMM d, yyyy, h:mm a')}
              </span>
            )}
          </div>
        </div>
      )}

      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 shrink-0 px-6 pt-2">
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
            <UseProfileTab userId={userId} />
          </TabsContent>
          <TabsContent value="settings" className="mt-0 h-full w-full p-6">
            <UseProfileTab userId={userId} />
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
