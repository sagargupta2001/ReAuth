import { AuthenticatorNode } from '@/features/flow-builder/nodes/AuthenticatorNode.tsx'
import { StartNode } from '@/features/flow-builder/nodes/StartNode.tsx'
import { TerminalNode } from '@/features/flow-builder/nodes/TerminalNode.tsx'

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
