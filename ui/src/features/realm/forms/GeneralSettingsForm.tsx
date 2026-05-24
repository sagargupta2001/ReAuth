import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'
import { toast } from 'sonner'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useApplyRecommendedPasskeyFlow } from '@/features/realm/api/useApplyRecommendedPasskeyFlow'
import { useApplyRecommendedPasskeyRegistrationFlow } from '@/features/realm/api/useApplyRecommendedPasskeyRegistrationFlow'
import { useRealmPasskeySettings } from '@/features/realm/api/useRealmPasskeySettings'
import { useUpdateRealm } from '@/features/realm/api/useUpdateRealm.ts'
import { useUpdateRealmPasskeySettings } from '@/features/realm/api/useUpdateRealmPasskeySettings'
import { useUpdateRealmOptimistic } from '@/features/realm/api/useUpdateRealmOptimistic'
import {
  type GeneralSettingsSchema,
  generalSettingsSchema,
} from '@/features/realm/schema/setting.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card.tsx'
import { Button } from '@/shared/ui/button'
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/shared/ui/form.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form } from '@/shared/ui/form.tsx'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import { Switch } from '@/shared/ui/switch'

export function GeneralSettingsForm() {
  const { data: realm, isLoading } = useCurrentRealm()
  const { data: passkeySettings, isLoading: isPasskeyLoading } = useRealmPasskeySettings()
  const updateMutation = useUpdateRealm(realm?.id || '')
  const toggleMutation = useUpdateRealmOptimistic(realm?.id || '', realm?.name || '')
  const updatePasskeyMutation = useUpdateRealmPasskeySettings(realm?.id || '')
  const recommendedFlowMutation = useApplyRecommendedPasskeyFlow(realm?.id || '')
  const recommendedRegistrationFlowMutation = useApplyRecommendedPasskeyRegistrationFlow(
    realm?.id || '',
  )

  const form = useForm<GeneralSettingsSchema>({
    resolver: zodResolver(generalSettingsSchema) as Resolver<GeneralSettingsSchema>,
    defaultValues: {
      name: '',
      invitation_resend_limit: 3,
      idp_broker_enabled: false,
      idp_default_jit_policy: 'per_provider',
      idp_default_email_link_policy: 'manual_only',
      idp_minimum_remaining_factor: true,
    },
  })

  const onSubmit = (values: GeneralSettingsSchema) => {
    updateMutation.mutate(values, {
      // RHF needs to know the form is "clean" after save
      onSuccess: (data) => form.reset(data),
    })
  }

  // Plug into the Global Bar
  useFormPersistence(form, onSubmit, updateMutation.isPending)

  useEffect(() => {
    if (realm)
      form.reset({
        name: realm.name,
        invitation_resend_limit: realm.invitation_resend_limit ?? 3,
        idp_broker_enabled: realm.idp_broker_enabled,
        idp_default_jit_policy: realm.idp_default_jit_policy,
        idp_default_email_link_policy: realm.idp_default_email_link_policy,
        idp_minimum_remaining_factor: realm.idp_minimum_remaining_factor,
      })
  }, [realm, form])

  if (isLoading || isPasskeyLoading) return null

  const registrationEnabled = Boolean(realm?.registration_enabled)
  const registrationBlocked = Boolean(realm?.is_system)
  const passkeysEnabled = Boolean(passkeySettings?.enabled)
  const handleRegistrationToggle = (enabled: boolean) => {
    if (!realm) return

    if (registrationBlocked) {
      toast.error('Self-registration cannot be enabled for the master realm.')
      return
    }

    if (enabled && !realm.registration_flow_id) {
      toast.error('No registration flow is configured for this realm.')
      return
    }

    toggleMutation.mutate({
      registration_enabled: enabled,
    })
  }

  const handlePasskeyToggle = (enabled: boolean) => {
    if (!realm?.id) return
    updatePasskeyMutation.mutate({
      enabled,
    })
  }

  const handleApplyRecommendedPasskeyFlow = () => {
    if (!realm?.id) return
    recommendedFlowMutation.mutate()
  }

  const handleApplyRecommendedRegistrationPasskeyFlow = () => {
    if (!realm?.id) return
    recommendedRegistrationFlowMutation.mutate()
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Card>
          <CardHeader>
            <CardTitle>Basic Settings</CardTitle>
            <CardDescription>The fundamental identity of your realm.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-6">
              <div id="realm-name" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="name"
                  label="Realm Name"
                  description="This appears in the URL. Changing this will redirect you."
                  placeholder="e.g. my-tenant"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <div id="realm-registration" className="scroll-mt-24 rounded-md -m-2 p-2">
          <Card className="mt-6">
            <CardHeader>
              <CardTitle>Registration</CardTitle>
              <CardDescription>Control whether self-service user registration is active.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <div className="text-sm font-medium">Enable User Registration</div>
                  <div className="text-xs text-muted-foreground">
                    {registrationBlocked
                      ? 'Master realm registration is always disabled.'
                      : 'Turn off to disable the registration flow for this realm.'}
                  </div>
                </div>
                <Switch
                  checked={registrationEnabled}
                  onCheckedChange={handleRegistrationToggle}
                  aria-label="Enable user registration"
                  disabled={toggleMutation.isPending || registrationBlocked}
                />
              </div>

              <div className="max-w-sm">
                <FormInput
                  control={form.control}
                  name="invitation_resend_limit"
                  label="Invitation Resend Limit"
                  type="number"
                  min={0}
                  description="Maximum number of resends allowed per invitation in this realm."
                />
              </div>
            </CardContent>
          </Card>
        </div>

        <div id="realm-identity-brokering" className="scroll-mt-24 rounded-md -m-2 p-2">
          <Card className="mt-6">
            <CardHeader>
              <CardTitle>Identity Brokering</CardTitle>
              <CardDescription>
                Control whether inbound OAuth and OIDC providers are available in this realm.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <FormField
                control={form.control}
                name="idp_broker_enabled"
                render={({ field }) => (
                  <FormItem className="flex items-center justify-between rounded-lg border p-4">
                    <div className="space-y-1">
                      <FormLabel>Enable Identity Brokering</FormLabel>
                      <FormDescription>
                        Turns provider buttons and OAuth callback handling on for this realm.
                      </FormDescription>
                    </div>
                    <FormControl>
                      <Switch checked={Boolean(field.value)} onCheckedChange={field.onChange} />
                    </FormControl>
                  </FormItem>
                )}
              />

              <div className="grid gap-6 md:grid-cols-2">
                <FormField
                  control={form.control}
                  name="idp_default_jit_policy"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Default JIT Provisioning</FormLabel>
                      <Select value={field.value} onValueChange={field.onChange}>
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="allow">Allow by default</SelectItem>
                          <SelectItem value="per_provider">Require per-provider opt-in</SelectItem>
                          <SelectItem value="deny">Deny by default</SelectItem>
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        Sets the default Just-In-Time provisioning policy for newly created identity providers.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="idp_default_email_link_policy"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Default Email Auto-Link Policy</FormLabel>
                      <Select value={field.value} onValueChange={field.onChange}>
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="allow_verified">Allow verified email matches</SelectItem>
                          <SelectItem value="manual_only">Require manual linking</SelectItem>
                          <SelectItem value="deny">Disable email matching</SelectItem>
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        Controls the default provider behavior when an upstream email matches an existing local user.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>

              <FormField
                control={form.control}
                name="idp_minimum_remaining_factor"
                render={({ field }) => (
                  <FormItem className="flex items-center justify-between rounded-lg border p-4">
                    <div className="space-y-1">
                      <FormLabel>Protect The Last Sign-In Method</FormLabel>
                      <FormDescription>
                        Prevents users from unlinking their final remaining password, passkey, or federated sign-in factor.
                      </FormDescription>
                    </div>
                    <FormControl>
                      <Switch checked={Boolean(field.value)} onCheckedChange={field.onChange} />
                    </FormControl>
                  </FormItem>
                )}
              />
            </CardContent>
          </Card>
        </div>

        <div id="realm-passkeys" className="scroll-mt-24 rounded-md -m-2 p-2">
          <Card className="mt-6">
            <CardHeader>
              <CardTitle>Passkeys</CardTitle>
              <CardDescription>
                Enable passkeys and optionally apply the recommended passkey-first browser flow.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <div className="text-sm font-medium">Enable Passkeys</div>
                  <div className="text-xs text-muted-foreground">
                    Allows passkey assertion and enrollment nodes to run in this realm.
                  </div>
                </div>
                <Switch
                  checked={passkeysEnabled}
                  onCheckedChange={handlePasskeyToggle}
                  aria-label="Enable passkeys"
                  disabled={updatePasskeyMutation.isPending}
                />
              </div>
              <div className="flex items-center justify-between border-t pt-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium">Recommended Browser Flow</div>
                  <div className="text-xs text-muted-foreground">
                    Replaces the realm browser flow with a passkey-first template and keeps password fallback.
                  </div>
                </div>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleApplyRecommendedPasskeyFlow}
                  disabled={recommendedFlowMutation.isPending}
                >
                  Apply Recommended Flow
                </Button>
              </div>
              <div className="flex items-center justify-between border-t pt-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium">Recommended Registration Flow</div>
                  <div className="text-xs text-muted-foreground">
                    Inserts passkey enrollment after account creation in the registration flow.
                  </div>
                </div>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleApplyRecommendedRegistrationPasskeyFlow}
                  disabled={recommendedRegistrationFlowMutation.isPending}
                >
                  Apply Registration Flow
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </form>
    </Form>
  )
}
