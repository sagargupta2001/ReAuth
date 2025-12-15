import { AuthenticatorNode } from '@/entities/flow/ui/nodes/AuthenticatorNode.tsx'
import { StartNode } from '@/entities/flow/ui/nodes/StartNode.tsx'
import { TerminalNode } from '@/entities/flow/ui/nodes/TerminalNode.tsx'

export const flowNodeTypes = {
  // --- LOGIC NODES ---
  'core.start': StartNode,
  'core.start.flow': StartNode,

  // --- AUTHENTICATORS (Workers) ---
  'core.auth.cookie': AuthenticatorNode,
  'core.auth.password': AuthenticatorNode,
  'core.auth.otp': AuthenticatorNode,
  'core.auth.webauthn': AuthenticatorNode,

  // --- TERMINALS ---
  'core.terminal.allow': TerminalNode,
  'core.terminal.deny': TerminalNode,

  // --- FALLBACKS ---
  authenticator: AuthenticatorNode,
  terminal: TerminalNode,
}
