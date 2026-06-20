import { AlertTriangle, ArrowUpRight, Link2, Loader2, Radio, Trash2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { useSetBreadcrumb } from '@/features/breadcrumb/model/useBreadcrumbStore'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useIdentityProviderActivity } from '@/features/identity-provider/api/useIdentityProviderActivity'
import { useDeleteIdentityProvider } from '@/features/identity-provider/api/useDeleteIdentityProvider'
import { useIdentityProvider } from '@/features/identity-provider/api/useIdentityProvider'
import { useIdentityProviderLinkedUsers } from '@/features/identity-provider/api/useIdentityProviderLinkedUsers'
import { IdentityProviderForm } from '@/features/identity-provider/forms/IdentityProviderForm'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/shared/ui/alert-dialog'
import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { Main } from '@/widgets/Layout/Main'

function formatActivityAction(action: string) {
  return action
    .replace(/^idp_/, '')
    .split('_')
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ')
}

function isFailureAction(action: string) {
  return (
    action.endsWith('_failure') ||
    action === 'idp_state_mismatch' ||
    action === 'idp_conflict_email_collision'
  )
}

export function EditIdentityProviderPage() {
  const { providerId } = useParams<{ providerId: string }>()
  const navigate = useRealmNavigate()
  const { data: provider, isLoading, isError } = useIdentityProvider(providerId || '')
  const { data: activity, isLoading: activityLoading } = useIdentityProviderActivity(providerId || '')
  const { data: linkedUsers, isLoading: linkedUsersLoading } = useIdentityProviderLinkedUsers(
    providerId || '',
  )
  const deleteProvider = useDeleteIdentityProvider(providerId || '')

  useSetBreadcrumb({ [providerId ?? '']: provider?.display_name || provider?.alias || '' })

  if (isLoading) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading identity provider...</p>
      </div>
    )
  }

  if (isError || !provider) {
    return (
      <div className="text-destructive flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Failed to load identity provider details.</p>
        <Button variant="outline" onClick={() => navigate('/identity-providers')}>
          Go Back
        </Button>
      </div>
    )
  }

  return (
    <Main className="flex flex-1 flex-col gap-6 p-12">
      <div>
        <h2 className="text-2xl font-bold tracking-tight">{provider.display_name}</h2>
        <p className="text-muted-foreground">
          Review runtime endpoints, login/linking policy, and button presentation for{' '}
          <span className="font-mono">{provider.alias}</span>.
        </p>
      </div>
      <IdentityProviderForm provider={provider} />
      <Card>
        <CardHeader>
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Radio className="h-4 w-4" />
                Recent Broker Activity
              </CardTitle>
              <CardDescription>
                Recent <span className="font-mono">idp_*</span> audit activity for{' '}
                <span className="font-mono">{provider.alias}</span>, with auth session correlation
                for troubleshooting.
              </CardDescription>
            </div>
            <RealmLink
              to={`/logs?tab=logs&range=24h&log_q=${encodeURIComponent(provider.alias)}`}
            >
              <Button variant="outline" size="sm" className="gap-2">
                <ArrowUpRight className="h-4 w-4" />
                Open Logs
              </Button>
            </RealmLink>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {activityLoading ? (
            <div className="text-muted-foreground text-sm">Loading recent broker activity...</div>
          ) : !activity ? (
            <div className="text-muted-foreground text-sm">Activity is unavailable right now.</div>
          ) : (
            <>
              <div className="grid gap-3 md:grid-cols-4">
                <div className="rounded-md border p-3">
                  <div className="text-muted-foreground text-xs">Events (24h)</div>
                  <div className="text-lg font-semibold">
                    {activity.summary.total_events_last_24h}
                  </div>
                </div>
                <div className="rounded-md border border-destructive/30 bg-destructive/5 p-3">
                  <div className="text-muted-foreground text-xs">Failures (24h)</div>
                  <div className="text-lg font-semibold">
                    {activity.summary.failures_last_24h}
                  </div>
                </div>
                <div className="rounded-md border p-3">
                  <div className="text-muted-foreground text-xs">Callback Success (24h)</div>
                  <div className="text-lg font-semibold">
                    {activity.summary.callback_success_last_24h}
                  </div>
                </div>
                <div className="rounded-md border p-3">
                  <div className="text-muted-foreground text-xs">Links / JIT (24h)</div>
                  <div className="text-lg font-semibold">
                    {activity.summary.links_last_24h + activity.summary.jit_provisioned_last_24h}
                  </div>
                </div>
              </div>

              {!activity.recent_events.length ? (
                <div className="text-muted-foreground text-sm">
                  No brokering audit events have been recorded for this provider yet.
                </div>
              ) : (
                activity.recent_events.map((event) => {
                  const failure = isFailureAction(event.action)
                  const sessionSearch = event.auth_session_id || provider.alias
                  return (
                    <div
                      key={event.audit_event_id}
                      className="flex items-start justify-between gap-4 rounded-md border p-3"
                    >
                      <div className="space-y-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge variant={failure ? 'destructive' : 'outline'}>
                            {formatActivityAction(event.action)}
                          </Badge>
                          {event.linked_via ? (
                            <Badge variant="secondary">{event.linked_via}</Badge>
                          ) : null}
                          {failure ? (
                            <span className="text-destructive flex items-center gap-1 text-xs">
                              <AlertTriangle className="h-3.5 w-3.5" />
                              Attention
                            </span>
                          ) : null}
                        </div>
                        <div className="text-muted-foreground text-xs">
                          {new Date(event.created_at).toLocaleString()}
                          {event.email ? ` | email: ${event.email}` : ''}
                          {event.subject ? ` | subject: ${event.subject}` : ''}
                        </div>
                        <div className="text-muted-foreground break-all font-mono text-xs">
                          auth_session_id: {event.auth_session_id ?? 'n/a'}
                        </div>
                        {event.message ? (
                          <div className="text-sm">{event.message}</div>
                        ) : null}
                      </div>
                      <div className="flex shrink-0 flex-col gap-2">
                        <RealmLink
                          to={`/logs?tab=logs&range=24h&log_q=${encodeURIComponent(sessionSearch)}`}
                        >
                          <Button variant="outline" size="sm">
                            Logs
                          </Button>
                        </RealmLink>
                        {event.user_id ? (
                          <RealmLink to={`/users/${event.user_id}/credentials`}>
                            <Button variant="outline" size="sm">
                              Open User
                            </Button>
                          </RealmLink>
                        ) : null}
                      </div>
                    </div>
                  )
                })
              )}
            </>
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Link2 className="h-4 w-4" />
            Linked Users
          </CardTitle>
          <CardDescription>
            Users in this realm currently linked to <span className="font-mono">{provider.alias}</span>.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {linkedUsersLoading ? (
            <div className="text-muted-foreground text-sm">Loading linked users...</div>
          ) : !linkedUsers?.length ? (
            <div className="text-muted-foreground text-sm">No users are linked to this provider.</div>
          ) : (
            linkedUsers.map((linkedUser) => (
              <div
                key={linkedUser.federated_identity_id}
                className="flex items-center justify-between rounded-md border p-3"
              >
                <div className="space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <div className="text-sm font-medium">{linkedUser.username}</div>
                    <Badge variant="outline">{linkedUser.linked_via}</Badge>
                  </div>
                  <div className="text-muted-foreground text-xs">
                    subject: {linkedUser.subject}
                    {linkedUser.external_email ? ` | external email: ${linkedUser.external_email}` : ''}
                    {linkedUser.email ? ` | local email: ${linkedUser.email}` : ''}
                  </div>
                  <div className="text-muted-foreground text-xs">
                    linked: {new Date(linkedUser.linked_at).toLocaleString()}
                    {linkedUser.last_provider_login_at
                      ? ` | provider last login: ${new Date(linkedUser.last_provider_login_at).toLocaleString()}`
                      : ''}
                    {linkedUser.last_user_sign_in_at
                      ? ` | user last sign-in: ${new Date(linkedUser.last_user_sign_in_at).toLocaleString()}`
                      : ''}
                  </div>
                </div>
                <RealmLink to={`/users/${linkedUser.user_id}/credentials`}>
                  <Button variant="outline" size="sm">
                    Open User
                  </Button>
                </RealmLink>
              </div>
            ))
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Danger Zone</CardTitle>
          <CardDescription>
            Delete this provider, or disable it automatically when linked identities still exist.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="destructive" className="gap-2">
                <Trash2 className="h-4 w-4" />
                Delete Provider
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Delete {provider.display_name}?</AlertDialogTitle>
                <AlertDialogDescription>
                  Standard delete removes the provider immediately when it has no linked identities.
                  If linked identities still exist, ReAuth will soft-delete it by disabling login and
                  preserving those links. Hard delete permanently removes the provider and any linked
                  identities that still reference it.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter className="flex flex-col gap-2 sm:flex-row sm:items-center">
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <Button
                  variant="outline"
                  onClick={() =>
                    deleteProvider.mutate(
                      { hard: false },
                      {
                        onSuccess: () => navigate('/identity-providers'),
                      },
                    )
                  }
                  disabled={deleteProvider.isPending}
                >
                  Delete Or Disable
                </Button>
                <AlertDialogAction asChild>
                  <Button
                    variant="destructive"
                    onClick={() =>
                      deleteProvider.mutate(
                        { hard: true },
                        {
                          onSuccess: () => navigate('/identity-providers'),
                        },
                      )
                    }
                    disabled={deleteProvider.isPending}
                  >
                    Hard Delete
                  </Button>
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </CardContent>
      </Card>
    </Main>
  )
}
