import { useState } from 'react'

import { History, Layout, Loader2, Settings } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { ThemeDetailsOverviewTab } from '@/features/theme/components/ThemeDetailsOverviewTab'
import { ThemeDetailsSettingsTab } from '@/features/theme/components/ThemeDetailsSettingsTab'
import { ThemeHeader } from '@/features/theme/components/ThemeHeader'
import { ThemeHistoryTab } from '@/features/theme/components/ThemeHistoryTab'
import { useTheme } from '@/features/theme/api/useTheme'

export function ThemeDetailsPage() {
  const { themeId } = useParams()
  const navigate = useRealmNavigate()
  const [activeTab, setActiveTab] = useState('overview')

  const { data, isLoading, isError } = useTheme(themeId)

  if (isLoading) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Theme...</p>
      </div>
    )
  }

  if (isError || !data) {
    return (
      <div className="text-destructive flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Failed to load theme details.</p>
        <Button variant="outline" onClick={() => navigate(-1)}>
          Go Back
        </Button>
      </div>
    )
  }

  return (
    <div className="bg-background flex h-full w-full flex-col">
      <ThemeHeader
        theme={data.theme}
        activeVersionNumber={data.active_version_number ?? null}
      />

      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 border-b px-6 pt-2">
          <TabsList className="gap-6 bg-transparent p-0">
            <TabsTrigger value="overview" className="tab-trigger-styles">
              <Layout className="mr-2 h-4 w-4" /> Overview
            </TabsTrigger>
            <TabsTrigger value="history" className="tab-trigger-styles">
              <History className="mr-2 h-4 w-4" /> Version History
            </TabsTrigger>
            <TabsTrigger value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="overview" className="mt-0 h-full w-full flex-1">
          <ThemeDetailsOverviewTab theme={data.theme} />
        </TabsContent>

        <TabsContent value="history" className="mt-0 flex-1 overflow-auto">
          <ThemeHistoryTab themeId={data.theme.id} activeVersionId={data.active_version_id} />
        </TabsContent>

        <TabsContent value="settings" className="mt-0 flex-1">
          <ThemeDetailsSettingsTab theme={data.theme} />
        </TabsContent>
      </Tabs>
    </div>
  )
}
