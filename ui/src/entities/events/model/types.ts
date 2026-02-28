export interface WebhookEndpoint {
  id: string
  realm_id: string
  name: string
  url: string
  http_method: string
  status: string
  signing_secret: string
  custom_headers: Record<string, string>
  description?: string | null
  consecutive_failures: number
  last_fired_at?: string | null
  last_failure_at?: string | null
  disabled_at?: string | null
  disabled_reason?: string | null
  created_at: string
  updated_at: string
}

export interface WebhookSubscription {
  endpoint_id: string
  event_type: string
  enabled: boolean
  created_at: string
}

export interface WebhookEndpointDetails {
  endpoint: WebhookEndpoint
  subscriptions: WebhookSubscription[]
}

export interface DeliveryLog {
  id: string
  event_id: string
  realm_id?: string | null
  target_type: string
  target_id: string
  event_type: string
  event_version: string
  attempt: number
  payload: string
  payload_compressed: boolean
  response_status?: number | null
  response_body?: string | null
  error?: string | null
  error_chain?: string | null
  latency_ms?: number | null
  delivered_at: string
}

export interface CreateWebhookPayload {
  name: string
  url: string
  description?: string | null
  signing_secret?: string | null
  custom_headers?: Record<string, string>
  http_method?: string
  subscriptions: string[]
}
