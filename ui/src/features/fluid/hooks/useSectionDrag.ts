import { useCallback, useMemo, useRef, useState } from 'react'
import type { DragEvent, KeyboardEvent } from 'react'

import { toast } from 'sonner'

import type { ThemeNode } from '@/entities/theme/model/types'
import {
  findNodePath,
  resolveDrop,
  type NodeDropIntent,
  type NodeLocation,
} from '@/features/fluid/lib/nodeUtils'
import { canAcceptChildren } from '@/features/fluid/model/blockCatalog'
import {
  MAX_NESTING_DEPTH,
  SECTION_DRAG_MIME_TYPE,
  dropIntentForOffset,
} from '@/features/fluid/model/sectionTree'

/** The row the pointer is currently over, and what dropping there would do. */
export interface SectionDropTarget {
  nodeId: string
  intent: NodeDropIntent
  isAllowed: boolean
}

export interface SectionDragHandlers {
  draggingNodeId: string | null
  dropTarget: SectionDropTarget | null
  onDragStart: (event: DragEvent<HTMLElement>, nodeId: string) => void
  onDragEnd: () => void
  /** Drop zones on a node row: top edge, bottom edge, and (containers) middle. */
  onRowDragOver: (
    event: DragEvent<HTMLElement>,
    nodeId: string,
    acceptsChildren: boolean,
  ) => void
  onRowDrop: (event: DragEvent<HTMLElement>, nodeId: string, acceptsChildren: boolean) => void
  /** The whole-row "drop inside" target an empty container renders. */
  onInsideDragOver: (event: DragEvent<HTMLElement>, nodeId: string) => void
  onInsideDrop: (event: DragEvent<HTMLElement>, nodeId: string) => void
  onDragLeave: (event: DragEvent<HTMLElement>, nodeId: string) => void
  /** Keyboard parity for indent, outdent, and reorder. */
  onRowKeyDown: (event: KeyboardEvent<HTMLElement>, nodeId: string) => void
  /** Commits a move expressed the same way a drop is. Returns whether it acted. */
  requestMove: (dragId: string, targetId: string | null, intent: NodeDropIntent) => boolean
}

const DROP_OPTIONS = {
  acceptsChildren: canAcceptChildren,
  maxDepth: MAX_NESTING_DEPTH,
}

/**
 * Drag, drop, and keyboard wiring for restructuring the sections tree.
 *
 * Validity lives here rather than in the rows because a row cannot answer
 * "would this drop create a cycle?" on its own — that needs the whole tree.
 * Rows render whatever `dropTarget` says and stay presentational.
 */
export function useSectionDrag(
  nodes: ThemeNode[],
  onMoveNode: (nodeId: string, location: NodeLocation) => void,
  onExpandNode?: (nodeId: string) => void,
): SectionDragHandlers {
  const [draggingNodeId, setDraggingNodeId] = useState<string | null>(null)
  const [dropTarget, setDropTarget] = useState<SectionDropTarget | null>(null)
  // `dataTransfer` payloads are unreadable during dragover, and state updates
  // lag the event stream, so the in-flight drag is mirrored in refs.
  const draggingRef = useRef<string | null>(null)
  // dragover fires continuously; without this the auto-expand would re-fire on
  // every pixel of movement over the same collapsed container.
  const autoExpandedRef = useRef<string | null>(null)
  const nodesRef = useRef(nodes)
  nodesRef.current = nodes

  const clear = useCallback(() => {
    draggingRef.current = null
    autoExpandedRef.current = null
    setDraggingNodeId(null)
    setDropTarget(null)
  }, [])

  const requestMove = useCallback(
    (dragId: string, targetId: string | null, intent: NodeDropIntent) => {
      const resolution = resolveDrop(nodesRef.current, { dragId, targetId, intent }, DROP_OPTIONS)
      if (!resolution.ok) {
        // A no-op is a drop that landed where the node already was. Reporting
        // it would be noise, and committing it would add an empty undo entry.
        if (resolution.reason !== 'no-op') {
          toast.error(resolution.message)
        }
        return false
      }
      onMoveNode(dragId, resolution.location)
      return true
    },
    [onMoveNode],
  )

  const previewDrop = useCallback(
    (event: DragEvent<HTMLElement>, nodeId: string, intent: NodeDropIntent) => {
      const dragId = draggingRef.current
      if (!dragId) return
      event.preventDefault()
      event.stopPropagation()

      const resolution = resolveDrop(
        nodesRef.current,
        { dragId, targetId: nodeId, intent },
        DROP_OPTIONS,
      )
      const isAllowed = resolution.ok
      // The drop still has to fire when it is not allowed: the not-allowed
      // cursor shows *that* it is refused, and only the drop can say why.
      event.dataTransfer.dropEffect = isAllowed ? 'move' : 'none'
      // Hovering a collapsed container opens it, so the drop is never blind.
      if (isAllowed && intent === 'inside' && autoExpandedRef.current !== nodeId) {
        autoExpandedRef.current = nodeId
        onExpandNode?.(nodeId)
      }
      setDropTarget((previous) => {
        const next = { nodeId, intent: resolution.ok ? resolution.intent : intent, isAllowed }
        if (
          previous &&
          previous.nodeId === next.nodeId &&
          previous.intent === next.intent &&
          previous.isAllowed === next.isAllowed
        ) {
          return previous
        }
        return next
      })
    },
    [onExpandNode],
  )

  const commitDrop = useCallback(
    (event: DragEvent<HTMLElement>, nodeId: string, intent: NodeDropIntent) => {
      event.preventDefault()
      event.stopPropagation()
      const dragId =
        event.dataTransfer.getData(SECTION_DRAG_MIME_TYPE) || draggingRef.current || ''
      clear()
      if (!dragId) return
      requestMove(dragId, nodeId, intent)
    },
    [clear, requestMove],
  )

  return useMemo(
    () => ({
      draggingNodeId,
      dropTarget,
      onDragStart: (event, nodeId) => {
        event.dataTransfer.setData(SECTION_DRAG_MIME_TYPE, nodeId)
        event.dataTransfer.effectAllowed = 'move'
        event.stopPropagation()
        draggingRef.current = nodeId
        setDraggingNodeId(nodeId)
      },
      onDragEnd: clear,
      onRowDragOver: (event, nodeId, acceptsChildren) => {
        const rect = event.currentTarget.getBoundingClientRect()
        const intent = dropIntentForOffset(
          event.clientY - rect.top,
          rect.height,
          acceptsChildren,
        )
        previewDrop(event, nodeId, intent)
      },
      onRowDrop: (event, nodeId, acceptsChildren) => {
        const pending = dropTarget?.nodeId === nodeId ? dropTarget.intent : null
        const rect = event.currentTarget.getBoundingClientRect()
        const intent =
          pending ??
          dropIntentForOffset(event.clientY - rect.top, rect.height, acceptsChildren)
        commitDrop(event, nodeId, intent)
      },
      onInsideDragOver: (event, nodeId) => previewDrop(event, nodeId, 'inside'),
      onInsideDrop: (event, nodeId) => commitDrop(event, nodeId, 'inside'),
      onDragLeave: (event, nodeId) => {
        event.stopPropagation()
        setDropTarget((previous) => (previous?.nodeId === nodeId ? null : previous))
      },
      onRowKeyDown: (event, nodeId) => {
        const isIndent = event.key === 'Tab' && !event.shiftKey
        const isOutdent = event.key === 'Tab' && event.shiftKey
        const isMoveUp = event.altKey && event.key === 'ArrowUp'
        const isMoveDown = event.altKey && event.key === 'ArrowDown'
        if (!isIndent && !isOutdent && !isMoveUp && !isMoveDown) return

        const path = findNodePath(nodesRef.current, nodeId)
        if (!path) return
        const parent = path.length > 1 ? path[path.length - 2] : null
        const siblings = parent ? parent.children ?? [] : nodesRef.current
        const index = siblings.findIndex((sibling) => sibling.id === nodeId)

        let target: string | null = null
        let intent: NodeDropIntent = 'inside'
        if (isIndent) {
          const previous = index > 0 ? siblings[index - 1] : undefined
          // Falling through to normal focus movement when there is nothing to
          // indent into is what keeps the tree escapable by keyboard.
          if (!previous || !canAcceptChildren(previous)) return
          target = previous.id
        } else if (isOutdent) {
          if (!parent) return
          target = parent.id
          intent = 'after'
        } else if (isMoveUp) {
          if (index <= 0) return
          target = siblings[index - 1].id
          intent = 'before'
        } else {
          if (index < 0 || index >= siblings.length - 1) return
          target = siblings[index + 1].id
          intent = 'after'
        }

        event.preventDefault()
        requestMove(nodeId, target, intent)
      },
      requestMove,
    }),
    [clear, commitDrop, draggingNodeId, dropTarget, previewDrop, requestMove],
  )
}
