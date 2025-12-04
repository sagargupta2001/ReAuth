import { Workflow } from 'lucide-react'

export function FlowsIndexPage() {
  return (
    <div className="flex h-full flex-col items-center justify-center space-y-4 text-center">
      <div className="bg-muted flex h-20 w-20 items-center justify-center rounded-full">
        <Workflow className="text-muted-foreground h-10 w-10" />
      </div>
      <div className="max-w-md space-y-2">
        <h2 className="text-2xl font-bold tracking-tight">Authentication Flows</h2>
        <p className="text-muted-foreground">
          Flows define the sequence of actions a user must take to authenticate. Select a flow from
          the sidebar to view or edit its execution plan.
        </p>
      </div>
    </div>
  )
}
