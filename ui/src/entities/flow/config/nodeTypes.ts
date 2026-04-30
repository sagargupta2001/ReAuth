import { AuthenticatorNode } from '@/features/flow-builder/nodes/AuthenticatorNode.tsx'
import { LogicNode } from '@/features/flow-builder/nodes/LogicNode.tsx'
import { StartNode } from '@/features/flow-builder/nodes/StartNode.tsx'
import { TerminalNode } from '@/features/flow-builder/nodes/TerminalNode.tsx'

export const flowNodeTypes = {
  // --- LOGIC NODES ---
  'core.start': StartNode,
  'core.logic.condition': LogicNode,
  'core.logic.recovery_issue': LogicNode,
  'core.logic.issue_email_otp': LogicNode,
  'core.logic.subflow': LogicNode,

  // --- AUTHENTICATORS (Workers) ---
  'core.auth.cookie': AuthenticatorNode,
  'core.auth.password': AuthenticatorNode,
  'core.auth.register': AuthenticatorNode,
  'core.auth.forgot_credentials': AuthenticatorNode,
  'core.auth.reset_password': AuthenticatorNode,
  'core.auth.verify_email_otp': AuthenticatorNode,
  'core.oidc.consent': AuthenticatorNode,

  // --- TERMINALS ---
  'core.terminal.allow': TerminalNode,
  'core.terminal.deny': TerminalNode,

  // --- FALLBACKS ---
  authenticator: AuthenticatorNode,
  terminal: TerminalNode,
}
