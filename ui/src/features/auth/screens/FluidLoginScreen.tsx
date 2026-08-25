import { useEffect, useMemo, useRef, useState } from 'react'
import type { ReactNode } from 'react'

import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { Form, FormField } from '@/components/form'
import { Input } from '@/components/input'
import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'
import { PasswordInput } from '@/shared/ui/password-input'
import { loginSchema } from '@/features/auth/schema/login.schema'
import type { AuthScreenProps } from '@/entities/auth/model/screenTypes'
import type { ThemeNode } from '@/entities/theme/model/types'
import { authApi } from '@/features/auth/api/authApi'
import { useThemeSnapshot } from '@/features/theme/api/useThemeSnapshot'
import { cn } from '@/lib/utils'
import { UsernamePasswordScreen } from '@/features/auth/screens/UsernamePasswordScreen'
import { FluidShell } from '@/features/fluid/components/FluidShell'
import { partitionShellBlocks } from '@/features/fluid/lib/shellBlocks'
import type { ComponentThemeContext } from '@/features/fluid/lib/componentRegistry'
import { getNestedRecord, resolveThemeColor } from '@/features/fluid/lib/themeUtils'
import { resolveVisibleFlag } from '@/features/fluid/lib/nodeVisuals'
import {
  renderFluidNode,
  type FluidHost,
  type FluidRenderOptions,
} from '@/features/fluid/lib/renderFluidNode'

type LoginFormValues = Record<string, string>
type IdentityProviderOption = {
  alias: string
  display_name: string
  icon_ref?: string | null
  button_color?: string | null
  sort_order?: number
}

type PasskeyRequestOptionsJson = {
  challenge: string
  timeout?: number
  rpId?: string
  userVerification?: UserVerificationRequirement
  allowCredentials?: Array<{
    type: PublicKeyCredentialType
    id: string
    transports?: AuthenticatorTransport[]
  }>
}

type PasskeyCreationOptionsJson = {
  challenge: string
  rp: {
    id?: string
    name: string
  }
  user: {
    id: string
    name: string
    displayName: string
  }
  pubKeyCredParams: Array<{
    type: PublicKeyCredentialType
    alg: number
  }>
  timeout?: number
  attestation?: AttestationConveyancePreference
  authenticatorSelection?: AuthenticatorSelectionCriteria
  excludeCredentials?: Array<{
    type: PublicKeyCredentialType
    id: string
    transports?: AuthenticatorTransport[]
  }>
}

const toBase64Url = (buffer: ArrayBuffer | null): string | null => {
  if (!buffer) return null
  const bytes = new Uint8Array(buffer)
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '')
}

const fromBase64Url = (value: string): Uint8Array => {
  const normalized = value.replace(/-/g, '+').replace(/_/g, '/')
  const padLength = (4 - (normalized.length % 4)) % 4
  const padded = normalized + '='.repeat(padLength)
  const decoded = atob(padded)
  const bytes = new Uint8Array(decoded.length)
  for (let i = 0; i < decoded.length; i += 1) {
    bytes[i] = decoded.charCodeAt(i)
  }
  return bytes
}

const toPublicKeyRequestOptions = (
  options: PasskeyRequestOptionsJson,
): PublicKeyCredentialRequestOptions => ({
  challenge: fromBase64Url(options.challenge),
  timeout: options.timeout,
  rpId: options.rpId,
  userVerification: options.userVerification,
  allowCredentials: options.allowCredentials?.map((credential) => ({
    type: credential.type,
    id: fromBase64Url(credential.id),
    transports: credential.transports,
  })),
})

const toPublicKeyCreationOptions = (
  options: PasskeyCreationOptionsJson,
): PublicKeyCredentialCreationOptions => ({
  challenge: fromBase64Url(options.challenge),
  rp: options.rp,
  user: {
    ...options.user,
    id: fromBase64Url(options.user.id),
  },
  pubKeyCredParams: options.pubKeyCredParams,
  timeout: options.timeout,
  attestation: options.attestation,
  authenticatorSelection: options.authenticatorSelection,
  excludeCredentials: options.excludeCredentials?.map((credential) => ({
    type: credential.type,
    id: fromBase64Url(credential.id),
    transports: credential.transports,
  })),
})


import {
  findActionInTree,
  nodeActions,
  normalizeTrigger,
  type FluidAction,
  } from '@/features/auth/lib/fluidLoginUtils'

export function FluidLoginScreen({
  onSubmit,
  isLoading,
  error,
  context,
  realm = 'master',
  clientId,
}: AuthScreenProps) {
  const templateKey =
    typeof context?.template_key === 'string' ? (context.template_key as string) : 'login'
  const activeRealm = realm ?? 'master'

  const resumeToken =
    typeof context?.resume_token === 'string' ? (context.resume_token as string) : null
  const resumePath =
    typeof context?.resume_path === 'string' ? (context.resume_path as string) : null
  const actionType =
    typeof context?.action_type === 'string' ? (context.action_type as string) : null
  const expiresAt =
    typeof context?.expires_at === 'string'
      ? (context.expires_at as string)
      : context?.expires_at instanceof Date
        ? context.expires_at.toISOString()
        : null
  const expiresAtDate = expiresAt ? new Date(expiresAt) : null
  const expiresInMinutes =
    expiresAtDate != null
      ? Math.max(0, Math.ceil((expiresAtDate.getTime() - Date.now()) / 60000))
      : null
  const isExpired = expiresAtDate ? expiresAtDate.getTime() <= Date.now() : false
  const canResend =
    Boolean(resumeToken) &&
    (actionType === 'reset_credentials' || actionType === 'email_verify')
  const [resendStatus, setResendStatus] = useState<'idle' | 'sending' | 'sent' | 'error'>(
    'idle',
  )
  const [payloadMapWarnings, setPayloadMapWarnings] = useState<string[]>([])
  const [payloadMapActionMeta, setPayloadMapActionMeta] = useState<{
    type?: string
    node_id?: string
  } | null>(null)
  const [autoStatus, setAutoStatus] = useState<'idle' | 'consumed' | 'expired' | 'error'>(
    'idle',
  )
  const pollDelayRef = useRef(2000)
  const pollTimeoutRef = useRef<number | null>(null)

  const { data: snapshot, isLoading: isThemeLoading } = useThemeSnapshot(realm, {
    pageKey: templateKey,
    clientId,
  })
  const [localError, setLocalError] = useState<string | null>(null)
  const [passkeyError, setPasskeyError] = useState<string | null>(null)
  const [isPasskeyBusy, setIsPasskeyBusy] = useState(false)

  const isPasskeyEnrollScreen = context?.passkey_enrollment === true
  const isPasskeyAssertScreen =
    context?.passkeys_enabled === true &&
    typeof context?.fallback_allowed === 'boolean' &&
    !isPasskeyEnrollScreen
  const passkeyIntent =
    context?.passkey_intent === 'reauth' || context?.passkey_intent === 'login'
      ? (context.passkey_intent as 'login' | 'reauth')
      : 'login'
  const fallbackAllowed = context?.fallback_allowed === true
  const canSkipEnrollment = context?.can_skip !== false
  const authSessionId =
    typeof context?.auth_session_id === 'string' ? (context.auth_session_id as string) : undefined
  const oauthProviderAlias =
    typeof context?.provider_alias === 'string' ? (context.provider_alias as string) : undefined
  const enabledProviders = useMemo<IdentityProviderOption[]>(
    () =>
      Array.isArray(context?.enabled_providers)
        ? (context.enabled_providers as IdentityProviderOption[])
        : [],
    [context?.enabled_providers],
  )

  const form = useForm<LoginFormValues>({
    defaultValues: {
      username: (context?.username as string) || '',
      password: '',
      otp: '',
      passkey_friendly_name: '',
    },
  })

  useEffect(() => {
    if (context?.username) {
      form.setValue('username', context.username as string)
    }
  }, [context?.username, form])

  useEffect(() => {
    if (templateKey === 'forgot_credentials' && context?.email) {
      form.setValue('email', context.email as string)
    }
  }, [context?.email, form, templateKey])

  useEffect(() => {
    setPasskeyError(null)
    setIsPasskeyBusy(false)
  }, [isPasskeyAssertScreen, isPasskeyEnrollScreen, templateKey])

  const displayError = passkeyError || localError || error || (context?.error as string) || null
  const showPayloadMapWarning = import.meta.env.DEV && payloadMapWarnings.length > 0
  const payloadMapWarningText = payloadMapWarnings.join(', ')
  const payloadMapActionLabel = payloadMapActionMeta
    ? `action=${payloadMapActionMeta.type ?? 'unknown'}${
        payloadMapActionMeta.node_id ? `, node_id=${payloadMapActionMeta.node_id}` : ''
      }`
    : null

  const awaitingStatusMessage = useMemo(() => {
    if (templateKey !== 'awaiting_action') return null
    if (autoStatus === 'consumed') return 'Recovery confirmed, redirecting…'
    if (autoStatus === 'expired') return 'Token expired. Request a new one.'
    if (autoStatus === 'error') return 'Waiting for confirmation…'
    return null
  }, [autoStatus, templateKey])

  const resendMessage =
    templateKey === 'awaiting_action'
      ? resendStatus === 'sent'
        ? 'Email sent.'
        : resendStatus === 'error'
          ? 'Unable to resend email.'
          : null
      : null

  const contextualValues = useMemo(() => {
    const base = typeof context === 'object' && context ? context : {}
    return {
      ...base,
      can_resend: canResend,
      awaiting_status: autoStatus,
      awaiting_status_message: awaitingStatusMessage,
      resend_message: resendMessage,
      expires_at: expiresAt,
      expires_in_minutes: expiresInMinutes,
      is_expired: isExpired,
    }
  }, [
    context,
    canResend,
    autoStatus,
    awaitingStatusMessage,
    resendMessage,
    expiresAt,
    expiresInMinutes,
    isExpired,
  ])

  const tokens = useMemo(() => snapshot?.tokens ?? {}, [snapshot])
  const layout = useMemo(() => snapshot?.layout ?? { shell: 'CenteredCard' }, [snapshot])
  const nodes = useMemo<ThemeNode[]>(() => snapshot?.nodes ?? [], [snapshot])
  const assets = useMemo(() => snapshot?.assets ?? [], [snapshot])

  const colors = getNestedRecord(tokens, 'colors')
  const typography = getNestedRecord(tokens, 'typography')
  const radius = getNestedRecord(tokens, 'radius')

  const rawBackground = String(colors.background || '')
  const rawText = String(colors.text || '')
  const rawPrimary = String(colors.primary || '')
  const rawSurface = String(colors.surface || '')
  const radiusBase = Number.parseFloat(String(radius.base || '12')) || 12
  const fontFamily = String(typography.font_family || 'system-ui')
  const baseSize = Number.parseFloat(String(typography.base_size || '16')) || 16
  const shell = typeof layout.shell === 'string' ? layout.shell : 'CenteredCard'
  const assetMap = new Map(assets.map((asset) => [asset.id, asset]))
  const background = resolveThemeColor(rawBackground, 'var(--background)')
  const text = resolveThemeColor(rawText, 'var(--foreground)')
  const primary = rawPrimary.trim() || 'var(--primary)'
  const surface = resolveThemeColor(rawSurface, 'var(--card)')
  const componentTheme: ComponentThemeContext = { text, radius: radiusBase }

  const {
    brand: brandBlocks,
    form: formBlocks,
    nonSplit: nonSplitBlocks,
  } = useMemo(() => partitionShellBlocks(nodes), [nodes])

  const submitAction = useMemo(() => findActionInTree(nodes, 'on_submit'), [nodes])

  const resolveValuePath = (
    source: Record<string, unknown>,
    path: string,
  ): unknown => {
    if (!path) return source
    const parts = path.split('.')
    let current: unknown = source
    for (const part of parts) {
      if (!part) continue
      if (!current || typeof current !== 'object') return undefined
      current = (current as Record<string, unknown>)[part]
    }
    return current
  }

  const resolvePayloadPath = (
    path: string,
    values: Record<string, unknown>,
  ): unknown => {
    const trimmed = path.trim()
    if (!trimmed) return undefined
    const parts = trimmed.split('.')
    const root = parts[0]
    const remainder = parts.slice(1).join('.')
    if (root === 'inputs') {
      return resolveValuePath(values, remainder)
    }
    if (root === 'context') {
      return resolveValuePath(contextualValues as Record<string, unknown>, remainder)
    }
    return undefined
  }

  const buildPayloadFromMap = (
    payloadMap: Record<string, unknown> | undefined,
    values: Record<string, unknown>,
    actionMeta?: { type?: string; node_id?: string },
  ) => {
    if (!payloadMap || Array.isArray(payloadMap) || typeof payloadMap !== 'object') {
      setPayloadMapWarnings([])
      setPayloadMapActionMeta(null)
      return { ...values }
    }
    const payload: Record<string, unknown> = {}
    const missing: Array<{ key: string; path: string }> = []
    for (const [key, mapping] of Object.entries(payloadMap)) {
      if (!key.trim() || typeof mapping !== 'string') continue
      const resolved = resolvePayloadPath(mapping, values)
      if (resolved === undefined) {
        missing.push({ key, path: mapping })
        continue
      }
      payload[key] = resolved
    }
    if (missing.length > 0) {
      setPayloadMapWarnings(missing.map((item) => `${item.key} <- ${item.path}`))
      setPayloadMapActionMeta(actionMeta ?? null)
      console.warn('[Fluid] payload_map unresolved values', {
        template_key: templateKey,
        action: actionMeta,
        missing,
      })
    } else if (payloadMap && typeof payloadMap === 'object') {
      setPayloadMapWarnings([])
      setPayloadMapActionMeta(null)
    }
    return payload
  }

  const buildSignalEnvelope = (
    action: FluidAction | null | undefined,
    values: Record<string, unknown>,
  ) => {
    const signal = action?.signal ?? {}
    const type =
      typeof signal.type === 'string' && signal.type.trim()
        ? signal.type.trim()
        : 'submit_node'
    const nodeId =
      typeof signal.node_id === 'string' && signal.node_id.trim()
        ? signal.node_id.trim()
        : undefined
    const payloadMap =
      signal.payload_map && typeof signal.payload_map === 'object'
        ? (signal.payload_map as Record<string, unknown>)
        : undefined
    const payload = buildPayloadFromMap(payloadMap, values, {
      type,
      node_id: nodeId,
    })

    return {
      signal: {
        type,
        node_id: nodeId,
        payload,
      },
    } as Record<string, unknown>
  }

  const processSubmission = (
    values: Record<string, string>,
    actionOverride?: FluidAction | null,
  ) => {
    if (templateKey === 'awaiting_action') {
      return
    }
    setLocalError(null)
    const normalized = { ...values }
    if (!normalized.username && normalized.email) {
      normalized.username = normalized.email
    }
    if (templateKey === 'forgot_credentials') {
      if (!normalized.username) {
        setLocalError('Email or username is required.')
        return
      }
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'mfa') {
      const otp =
        normalized.otp || normalized.code || normalized.token || normalized.verification_code
      if (!otp) {
        setLocalError('Verification code is required.')
        return
      }
      void onSubmit({ otp })
      return
    }
    if (templateKey === 'verify_email') {
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'reset_password') {
      const minLength =
        typeof context?.min_password_length === 'number'
          ? context.min_password_length
          : 8
      if (!normalized.password) {
        setLocalError('Password is required.')
        return
      }
      if (String(normalized.password).length < minLength) {
        setLocalError(`Password must be at least ${minLength} characters.`)
        return
      }
      const confirm =
        normalized.password_confirm ||
        normalized.confirm_password ||
        normalized.password_confirmation
      if (confirm && confirm !== normalized.password) {
        setLocalError('Passwords do not match.')
        return
      }
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'consent') {
      if (!normalized.decision) {
        setLocalError('Select allow or deny to continue.')
        return
      }
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'oauth_select') {
      if (normalized.decision === 'cancel') {
        void onSubmit({ decision: 'cancel' })
        return
      }
      if (!normalized.provider_alias) {
        setLocalError('Choose an identity provider to continue.')
        return
      }
      void onSubmit({ provider_alias: normalized.provider_alias })
      return
    }
    if (templateKey === 'oauth_link_confirm') {
      if (normalized.decision === 'cancel') {
        void onSubmit({ decision: 'cancel' })
        return
      }
      const parsed = loginSchema.safeParse(normalized)
      if (!parsed.success) {
        setLocalError(parsed.error.issues[0]?.message ?? 'Enter your local credentials.')
        return
      }
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'oauth_conflict' || templateKey === 'oauth_failure') {
      if (normalized.provider_alias) {
        void onSubmit({ provider_alias: normalized.provider_alias })
        return
      }
      if (normalized.decision) {
        void onSubmit({ decision: normalized.decision })
        return
      }
      setLocalError('Choose how you want to continue.')
      return
    }
    const parsed = loginSchema.safeParse(normalized)
    if (!parsed.success) {
      setLocalError(parsed.error.issues[0]?.message ?? 'Invalid login details.')
      return
    }
    const action = actionOverride ?? submitAction
    if (action) {
      void onSubmit(buildSignalEnvelope(action, normalized))
      return
    }
    void onSubmit(normalized)
  }

  const handleSubmit = form.handleSubmit((values) => {
    processSubmission(values)
  })

  const resolveContextValue = (path: string): unknown => {
    const trimmed = path.trim()
    if (!trimmed) return undefined
    const parts = trimmed.split('.')
    let current: unknown = contextualValues
    for (const part of parts) {
      if (!part) continue
      if (!current || typeof current !== 'object') return undefined
      current = (current as Record<string, unknown>)[part]
    }
    return current
  }

  const coerceVisible = (value: unknown): boolean => {
    if (value === undefined || value === null) return false
    if (typeof value === 'boolean') return value
    if (typeof value === 'number') return value !== 0
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase()
      if (!normalized || normalized === 'false' || normalized === '0') return false
      if (normalized === 'true') return true
      return true
    }
    return Boolean(value)
  }

  const resolveVisibleIf = (value: unknown): boolean => {
    if (value === undefined) return true
    if (typeof value === 'boolean') return value
    if (typeof value === 'string') {
      const trimmed = value.trim()
      if (!trimmed) return true
      const lowered = trimmed.toLowerCase()
      if (lowered === 'true') return true
      if (lowered === 'false') return false
      return coerceVisible(resolveContextValue(trimmed))
    }
    return Boolean(value)
  }

  const handleResend = async () => {
    if (!resumeToken || !canResend) return
    setResendStatus('sending')
    try {
      await authApi.resendAction(activeRealm, resumeToken)
      setResendStatus('sent')
    } catch (error) {
      console.error('[AwaitingAction] Resend failed', error)
      setResendStatus('error')
    }
  }

  useEffect(() => {
    if (templateKey !== 'awaiting_action') return
    setAutoStatus('idle')
    setResendStatus('idle')
    pollDelayRef.current = 2000
  }, [templateKey, resumeToken])

  useEffect(() => {
    if (templateKey !== 'awaiting_action') return
    if (!resumeToken || !resumePath) return
    if (autoStatus === 'consumed' || autoStatus === 'expired') return
    let cancelled = false

    const buildResumeRedirectUrl = () => {
      const [path, rawQuery] = resumePath.split('?')
      const params = new URLSearchParams(rawQuery || '')
      params.set('realm', activeRealm)
      params.set('resume', 'consumed')
      params.set('ts', Date.now().toString())
      const query = params.toString()
      return `/#${path}${query ? `?${query}` : ''}`
    }

    const poll = async () => {
      try {
        const response = await authApi.actionStatus(activeRealm, resumeToken)
        if (cancelled) return
        if (response.status === 'consumed') {
          setAutoStatus('consumed')
          const redirectUrl = buildResumeRedirectUrl()
          window.setTimeout(() => {
            window.location.href = redirectUrl
          }, 1200)
          return
        }
        if (response.status === 'expired') {
          setAutoStatus('expired')
          return
        }
        if (autoStatus !== 'idle') {
          setAutoStatus('idle')
        }
      } catch {
        if (!cancelled) {
          setAutoStatus('error')
        }
      }

      const nextDelay = Math.min(10000, Math.round(pollDelayRef.current * 1.5))
      pollDelayRef.current = nextDelay
      pollTimeoutRef.current = window.setTimeout(poll, pollDelayRef.current)
    }

    pollTimeoutRef.current = window.setTimeout(poll, pollDelayRef.current)
    return () => {
      cancelled = true
      if (pollTimeoutRef.current) {
        window.clearTimeout(pollTimeoutRef.current)
      }
    }
  }, [templateKey, resumeToken, resumePath, activeRealm, autoStatus])

  useEffect(() => {
    if (templateKey !== 'oauth_redirecting') return
    if (!oauthProviderAlias) return
    if (context?.auto_start !== true) return
    let cancelled = false

    const start = async () => {
      try {
        const response = await authApi.startOauth(activeRealm, oauthProviderAlias)
        if (!cancelled) {
          window.location.href = response.redirect_url
        }
      } catch (error) {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : 'OAuth redirect failed.'
          setLocalError(message)
        }
      }
    }

    void start()
    return () => {
      cancelled = true
    }
  }, [templateKey, oauthProviderAlias, activeRealm, context?.auto_start])

  const handlePasskeyFallback = () => {
    setPasskeyError(null)
    void onSubmit({ action: 'fallback' })
  }

  const handlePasskeyAuthentication = async () => {
    if (!isPasskeyAssertScreen) return

    if (
      typeof window === 'undefined' ||
      !('PublicKeyCredential' in window) ||
      !navigator.credentials?.get
    ) {
      setPasskeyError('Passkeys are not available in this browser.')
      return
    }

    setPasskeyError(null)
    setLocalError(null)
    setIsPasskeyBusy(true)

    try {
      const identifier = form.getValues('username')?.trim() || undefined
      const options = await authApi.passkeyAuthenticateOptions(activeRealm, {
        auth_session_id: authSessionId,
        identifier,
        intent: passkeyIntent,
      })
      const optionAllowCredentials = Array.isArray(
        (options.public_key as Record<string, unknown>)?.allowCredentials,
      )
        ? ((options.public_key as Record<string, unknown>).allowCredentials as unknown[])
        : []
      if (identifier && optionAllowCredentials.length === 0 && options.fallback_allowed) {
        setPasskeyError('No passkey is enrolled for this account. Continue with password.')
        void onSubmit({ action: 'fallback' })
        return
      }

      const parser = (
        window.PublicKeyCredential as unknown as {
          parseRequestOptionsFromJSON?: (
            json: PasskeyRequestOptionsJson,
          ) => PublicKeyCredentialRequestOptions
        }
      ).parseRequestOptionsFromJSON
      const publicKey =
        typeof parser === 'function'
          ? parser(options.public_key as PasskeyRequestOptionsJson)
          : toPublicKeyRequestOptions(options.public_key as PasskeyRequestOptionsJson)

      const credential = (await navigator.credentials.get({
        publicKey,
      })) as PublicKeyCredential | null

      if (!credential) {
        if (fallbackAllowed) {
          setPasskeyError('No passkey selected. Continue with password.')
        } else {
          setPasskeyError('No passkey selected.')
        }
        return
      }

      const assertion = credential.response as AuthenticatorAssertionResponse
      const verifyResponse = await authApi.passkeyAuthenticateVerify(activeRealm, {
        challenge_id: options.challenge_id,
        credential: {
          id: credential.id,
          rawId: toBase64Url(credential.rawId),
          type: credential.type,
          response: {
            clientDataJSON: toBase64Url(assertion.clientDataJSON),
            authenticatorData: toBase64Url(assertion.authenticatorData),
            signature: toBase64Url(assertion.signature),
            userHandle: toBase64Url(assertion.userHandle),
          },
        },
      })

      void onSubmit(verifyResponse as unknown as Record<string, unknown>)
    } catch (error) {
      if (error instanceof DOMException && error.name === 'NotAllowedError') {
        if (fallbackAllowed) {
          setPasskeyError('Passkey request was cancelled. Use password fallback to continue.')
        } else {
          setPasskeyError('Passkey request was cancelled.')
        }
      } else {
        const message = error instanceof Error ? error.message : 'Passkey authentication failed.'
        setPasskeyError(message)
      }
    } finally {
      setIsPasskeyBusy(false)
    }
  }

  const handlePasskeyEnrollmentSkip = () => {
    setPasskeyError(null)
    void onSubmit({ action: 'skip' })
  }

  const handlePasskeyEnrollment = async () => {
    if (!isPasskeyEnrollScreen) return
    if (
      typeof window === 'undefined' ||
      !('PublicKeyCredential' in window) ||
      !navigator.credentials?.create
    ) {
      setPasskeyError('Passkey enrollment is not available in this browser.')
      return
    }

    setPasskeyError(null)
    setLocalError(null)
    setIsPasskeyBusy(true)

    try {
      const options = await authApi.passkeyEnrollOptions(activeRealm, {
        auth_session_id: authSessionId,
      })
      const parser = (
        window.PublicKeyCredential as unknown as {
          parseCreationOptionsFromJSON?: (
            json: PasskeyCreationOptionsJson,
          ) => PublicKeyCredentialCreationOptions
        }
      ).parseCreationOptionsFromJSON
      const publicKey =
        typeof parser === 'function'
          ? parser(options.public_key as PasskeyCreationOptionsJson)
          : toPublicKeyCreationOptions(options.public_key as PasskeyCreationOptionsJson)

      const credential = (await navigator.credentials.create({
        publicKey,
      })) as PublicKeyCredential | null

      if (!credential) {
        setPasskeyError('Passkey enrollment was cancelled.')
        return
      }

      const attestation = credential.response as AuthenticatorAttestationResponse
      const authenticatorData = attestation.getAuthenticatorData?.() ?? null
      const publicKeyDer = attestation.getPublicKey?.() ?? null
      const publicKeyAlgorithm = attestation.getPublicKeyAlgorithm?.()
      const transports = attestation.getTransports?.() ?? []

      const verifyResponse = await authApi.passkeyEnrollVerify(activeRealm, {
        challenge_id: options.challenge_id,
        credential: {
          id: credential.id,
          rawId: toBase64Url(credential.rawId),
          type: credential.type,
          response: {
            clientDataJSON: toBase64Url(attestation.clientDataJSON),
            attestationObject: toBase64Url(attestation.attestationObject),
            authenticatorData: toBase64Url(authenticatorData),
            publicKey: toBase64Url(publicKeyDer),
            publicKeyAlgorithm:
              typeof publicKeyAlgorithm === 'number' ? publicKeyAlgorithm : undefined,
            transports,
          },
        },
        friendly_name: form.getValues('passkey_friendly_name')?.trim() || undefined,
      })

      void onSubmit(verifyResponse as unknown as Record<string, unknown>)
    } catch (error) {
      if (error instanceof DOMException && error.name === 'NotAllowedError') {
        setPasskeyError('Passkey enrollment was cancelled.')
      } else {
        const message = error instanceof Error ? error.message : 'Passkey enrollment failed.'
        setPasskeyError(message)
      }
    } finally {
      setIsPasskeyBusy(false)
    }
  }

  /**
   * The runtime half of the shared renderer: visibility-gated, form-wired, and
   * carrying actions, OAuth, and passkeys. Everything structural lives in
   * `renderFluidNode`, which `FluidCanvas` walks with its own host.
   */
  const host: FluidHost = {
    primary,
    componentTheme,
    assets: assetMap,
    isVisible: (node) => {
      const props = node.props ?? {}
      // Runtime-only gate: the builder shows every node so it stays editable.
      return resolveVisibleFlag(props.visible) && resolveVisibleIf(props.visible_if)
    },
    wrap: ({ node, index, content, className, visuals }) => (
      <div
        key={node.id ? `node-${node.id}` : `block-${index}`}
        className={cn(visuals.widthClass, visuals.heightClass, className)}
        style={visuals.style}
      >
        {content}
      </div>
    ),
    renderText: (_node, visuals) => {
      const textPath = String(visuals.props.text_path || '').trim()
      const resolved = textPath ? resolveContextValue(textPath) : undefined
      const textValue = resolved ?? visuals.props.text ?? 'Headline'
      // The wrapper already carries font_size/font_weight/color from props, so
      // only apply the heading defaults when the node does not set them —
      // a utility class here would override the inherited inline style.
      return (
        <p className={cn(!visuals.fontSize && 'text-lg', !visuals.fontWeight && 'font-semibold')}>
          {String(textValue)}
        </p>
      )
    },
    renderInput: (_node, { name, inputType, placeholder, inputClass }) => {
      const control =
        inputType === 'password' ? (
          <PasswordInput
            className="flex-1"
            inputClassName={inputClass}
            placeholder={placeholder}
            disabled={isLoading}
          />
        ) : (
          <Input
            className={inputClass}
            placeholder={placeholder}
            type={inputType}
            disabled={isLoading}
          />
        )
      if (!name) return control
      return (
        <FormField
          control={form.control}
          name={name}
          render={({ field }) =>
            inputType === 'password' ? (
              <PasswordInput
                {...field}
                className="flex-1"
                inputClassName={inputClass}
                placeholder={placeholder}
                disabled={isLoading}
              />
            ) : (
              <Input
                {...field}
                className={inputClass}
                placeholder={placeholder}
                type={inputType}
                disabled={isLoading}
              />
            )
          }
        />
      )
    },
    renderButton: (node, { defaultLabel, buttonVariant, className, style }) => {
      const props = node.props ?? {}
      const intent = typeof props.intent === 'string' ? props.intent.trim() : ''
      const clickAction = nodeActions(node).find(
        (action) => normalizeTrigger(action.trigger) === 'on_click' && action.signal,
      )
      const hasClickAction = Boolean(clickAction)
      const isAwaitingResend =
        templateKey === 'awaiting_action' && intent.toLowerCase() === 'resend'
      const label =
        isAwaitingResend && resendStatus === 'sending' ? 'Sending…' : defaultLabel
      return (
        <Button
          type={isAwaitingResend || hasClickAction ? 'button' : 'submit'}
          variant={buttonVariant}
          className={className}
          style={style}
          data-intent={intent || undefined}
          disabled={
            isLoading || (isAwaitingResend && (resendStatus === 'sending' || !canResend))
          }
          onClick={(event) => {
            if (templateKey === 'oauth_redirecting' && oauthProviderAlias) {
              event.preventDefault()
              void authApi.startOauth(activeRealm, oauthProviderAlias).then((response) => {
                window.location.href = response.redirect_url
              })
              return
            }
            if (isAwaitingResend) {
              void handleResend()
              return
            }
            if (intent) {
              form.setValue('decision', intent)
            }
            if (clickAction) {
              event.preventDefault()
              event.stopPropagation()
              void form
                .handleSubmit((values) => {
                  processSubmission(values, clickAction)
                })()
            }
          }}
        >
          {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          {label}
        </Button>
      )
    },
    renderProviders: () => {
      // Hiding the block outright is the runtime behaviour; the builder shows a
      // placeholder instead so the node stays selectable.
      if (enabledProviders.length === 0) return null
      return (
        <div className="flex w-full flex-col gap-3">
          {enabledProviders.map((provider) => {
            const accent = provider.button_color || primary
            return (
              <Button
                key={provider.alias}
                type="button"
                variant="outline"
                className="w-full justify-center"
                style={{ borderColor: accent, color: accent }}
                disabled={isLoading}
                onClick={() => {
                  setLocalError(null)
                  if (templateKey === 'login') {
                    void authApi
                      .startOauth(activeRealm, provider.alias)
                      .then((response) => {
                        window.location.href = response.redirect_url
                      })
                      .catch((error) => {
                        const message =
                          error instanceof Error
                            ? error.message
                            : 'Unable to start external sign-in.'
                        setLocalError(message)
                      })
                    return
                  }
                  void onSubmit({ provider_alias: provider.alias })
                }}
              >
                {provider.display_name}
              </Button>
            )
          })}
        </div>
      )
    },
  }

  const renderNode = (node: ThemeNode, index: number, options?: FluidRenderOptions): ReactNode =>
    renderFluidNode(node, host, index, options)

  if (isThemeLoading && !snapshot) {
    return (
      <div className="flex justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    )
  }

  if (isPasskeyEnrollScreen) {
    return (
      <div className="min-h-svh w-full" style={{ backgroundColor: background, color: text }}>
        <div className="flex min-h-svh w-full items-center justify-center p-8">
          <div
            className="w-full max-w-md border p-8 shadow-lg"
            style={{
              borderRadius: `${radiusBase}px`,
              backgroundColor: surface,
              color: text,
            }}
          >
            <Form {...form}>
              <form
                onSubmit={(event) => {
                  event.preventDefault()
                  void handlePasskeyEnrollment()
                }}
                className="space-y-4"
              >
                <div className="space-y-1">
                  <h1 className="text-xl font-semibold">Create a passkey</h1>
                  <p className="text-muted-foreground text-sm">
                    Secure your account with a device passkey for faster future sign-ins.
                  </p>
                </div>
                {displayError ? (
                  <div className="text-destructive rounded-md bg-red-50 p-3 text-sm font-medium">
                    {String(displayError)}
                  </div>
                ) : null}
                <FormField
                  control={form.control}
                  name="passkey_friendly_name"
                  render={({ field }) => (
                    <Input
                      {...field}
                      placeholder="Passkey label (optional)"
                      disabled={isLoading || isPasskeyBusy}
                    />
                  )}
                />
                <Button type="submit" className="w-full" disabled={isLoading || isPasskeyBusy}>
                  {isPasskeyBusy ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  Create passkey
                </Button>
                {canSkipEnrollment ? (
                  <Button
                    type="button"
                    variant="outline"
                    className="w-full"
                    disabled={isLoading || isPasskeyBusy}
                    onClick={handlePasskeyEnrollmentSkip}
                  >
                    Skip for now
                  </Button>
                ) : null}
              </form>
            </Form>
          </div>
        </div>
      </div>
    )
  }

  if (isPasskeyAssertScreen) {
    return (
      <div className="min-h-svh w-full" style={{ backgroundColor: background, color: text }}>
        <div className="flex min-h-svh w-full items-center justify-center p-8">
          <div
            className="w-full max-w-md border p-8 shadow-lg"
            style={{
              borderRadius: `${radiusBase}px`,
              backgroundColor: surface,
              color: text,
            }}
          >
            <Form {...form}>
              <form
                onSubmit={(event) => {
                  event.preventDefault()
                  void handlePasskeyAuthentication()
                }}
                className="space-y-4"
              >
                <div className="space-y-1">
                  <h1 className="text-xl font-semibold">Sign in with a passkey</h1>
                  <p className="text-muted-foreground text-sm">
                    Use your device passkey first. If unavailable, continue with password fallback.
                  </p>
                </div>
                {displayError ? (
                  <div className="text-destructive rounded-md bg-red-50 p-3 text-sm font-medium">
                    {String(displayError)}
                  </div>
                ) : null}
                <FormField
                  control={form.control}
                  name="username"
                  render={({ field }) => (
                    <Input
                      {...field}
                      placeholder="Email or username (optional)"
                      disabled={isLoading || isPasskeyBusy}
                    />
                  )}
                />
                <Button type="submit" className="w-full" disabled={isLoading || isPasskeyBusy}>
                  {isPasskeyBusy ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  Continue with passkey
                </Button>
                {fallbackAllowed ? (
                  <Button
                    type="button"
                    variant="outline"
                    className="w-full"
                    disabled={isLoading || isPasskeyBusy}
                    onClick={handlePasskeyFallback}
                  >
                    Use password instead
                  </Button>
                ) : null}
              </form>
            </Form>
          </div>
        </div>
      </div>
    )
  }

  if (!snapshot) {
    return (
      <UsernamePasswordScreen
        onSubmit={onSubmit}
        isLoading={isLoading}
        error={error}
        context={context}
        realm={realm}
        clientId={clientId}
      />
    )
  }

  const containerStyle: React.CSSProperties = {
    backgroundColor: background,
    color: text,
    fontFamily,
    fontSize: `${baseSize}px`,
  }

  const paneBlocks = shell === 'SplitScreen' ? formBlocks : nonSplitBlocks

  return (
    <div className="min-h-svh w-full" style={containerStyle}>
      <div className="flex min-h-svh w-full items-center justify-center p-8">
        <FluidShell
          shell={shell}
          surface={surface}
          background={background}
          text={text}
          radiusBase={radiusBase}
          hasBrand={brandBlocks.length > 0}
          brand={brandBlocks.map((block, index) => (
            <div key={`brand-${block.id ?? index}`} className="text-white">
              {renderNode(block, index, { wrapperClass: 'text-white' })}
            </div>
          ))}
        >
          <Form {...form}>
            <form onSubmit={handleSubmit} className="space-y-4">
              {templateKey === 'consent' ? (
                <input type="hidden" {...form.register('decision')} />
              ) : null}
              {displayError && (
                <div className="text-destructive mb-2 rounded-md bg-red-50 p-3 text-sm font-medium">
                  {String(displayError)}
                </div>
              )}
              {showPayloadMapWarning && (
                <Alert className="mb-2 border-amber-200 bg-amber-50 text-amber-900">
                  <AlertTitle>Developer warning</AlertTitle>
                  <AlertDescription>
                    payload_map unresolved: {payloadMapWarningText}
                    {payloadMapActionLabel ? ` (${payloadMapActionLabel})` : ''}
                  </AlertDescription>
                </Alert>
              )}
              {paneBlocks.length === 0 ? (
                <div className="text-muted-foreground text-sm">
                  Add blocks to build this page.
                </div>
              ) : (
                paneBlocks.map((block, index) => renderNode(block, index))
              )}
            </form>
          </Form>
        </FluidShell>
      </div>
    </div>
  )
}
