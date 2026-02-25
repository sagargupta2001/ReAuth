import { useMemo, useState } from 'react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/dialog'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useDeleteWebhook } from '@/features/events/api/useDeleteWebhook'
import { useWebhook } from '@/features/events/api/useWebhooks'
import { WebhookEndpointForm } from '@/features/events/components/WebhookEndpointForm'
import { Main } from '@/widgets/Layout/Main'
import { Trash2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

export function WebhookSettingsPage() {
  const { targetId } = useParams<{ targetId: string }>()
  const navigate = useRealmNavigate()
  const [deleteOpen, setDeleteOpen] = useState(false)

  const { data, isLoading, isError } = useWebhook(targetId)
  const deleteWebhook = useDeleteWebhook()

  const endpoint = data?.endpoint
  const subscriptions = useMemo(() => data?.subscriptions ?? [], [data?.subscriptions])
  const enabledSubscriptions = useMemo(
    () => subscriptions.filter((sub) => sub.enabled),
    [subscriptions],
  )

  const handleDelete = async () => {
    if (!endpoint) return
    try {
      await deleteWebhook.mutateAsync(endpoint.id)
      setDeleteOpen(false)
      navigate('/events')
    } catch (err) {
      console.error('Failed to delete webhook', err)
    }
  }

  return (
    <Main className="flex flex-1 flex-col gap-6 p-12" fixed>
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Webhook Settings</h1>
          <p className="text-sm text-muted-foreground">
            Update endpoint details, subscriptions, and manage lifecycle actions.
          </p>
        </div>
        <Button variant="outline" onClick={() => navigate('/events')}>
          Back to Event Routing
        </Button>
      </div>

      {isLoading ? (
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Loading webhook settings...
          </CardContent>
        </Card>
      ) : isError || !endpoint ? (
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Unable to load webhook settings.
          </CardContent>
        </Card>
      ) : (
        <>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>{endpoint.name}</CardTitle>
                <p className="text-sm text-muted-foreground">{endpoint.url}</p>
              </div>
              <Badge variant={endpoint.status === 'active' ? 'success' : 'destructive'}>
                {endpoint.status === 'active' ? 'Active' : 'Disabled'}
              </Badge>
            </CardHeader>
            <CardContent className="flex flex-wrap items-center justify-between gap-4">
              <div>
                <p className="text-sm font-semibold">Subscriptions</p>
                <p className="text-xs text-muted-foreground">
                  {enabledSubscriptions.length} events configured
                </p>
              </div>
              <WebhookEndpointForm
                mode="edit"
                endpointId={endpoint.id}
                initialUrl={endpoint.url}
                initialDescription={endpoint.description}
                initialSubscriptions={enabledSubscriptions.map((sub) => sub.event_type)}
                trigger={<Button>Edit Endpoint</Button>}
                onSaved={() => navigate(`/events/webhooks/${endpoint.id}/settings`)}
              />
            </CardContent>
          </Card>

          <Card className="border-destructive/30">
            <CardHeader>
              <CardTitle className="text-base text-destructive">Danger Zone</CardTitle>
            </CardHeader>
            <CardContent className="flex flex-wrap items-center justify-between gap-4">
              <div>
                <p className="text-sm font-semibold">Delete Webhook</p>
                <p className="text-xs text-muted-foreground">
                  This action permanently removes the endpoint and delivery history.
                </p>
              </div>
              <Button variant="destructive" onClick={() => setDeleteOpen(true)}>
                <Trash2 className="h-4 w-4" />
                Delete Endpoint
              </Button>
            </CardContent>
          </Card>
        </>
      )}

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete webhook endpoint</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">
            This will permanently delete the webhook and its subscriptions. This action cannot be
            undone.
          </p>
          <DialogFooter className="gap-2">
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={deleteWebhook.isPending}
            >
              {deleteWebhook.isPending ? 'Deleting...' : 'Delete Endpoint'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Main>
  )
}

export default WebhookSettingsPage
