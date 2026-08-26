import { useCallback, useMemo, useRef, useState } from 'react'
import type { DragEvent, KeyboardEvent } from 'react'

import { toast } from 'sonner'

import type { ThemeNode } from '@/entities/theme/model/types'
import {
  findNodePath,
  resolveDrop,
  resolveInsertion,
  subtreeHeight,
  type NodeDropIntent,
  type NodeLocation,
} from '@/features/fluid/lib/nodeUtils'
import {
  buildFluidNode,
  canAcceptChildren,
  findBlockDefinition,
  type FluidBlockId,
} from '@/features/fluid/model/blockCatalog'
import { dropIntentForOffset } from '@/features/fluid/model/dropZones'
import {
  BLOCK_DRAG_MIME_TYPE,
  MAX_NESTING_DEPTH,
  SECTION_DRAG_MIME_TYPE,
} from '@/features/fluid/model/sectionTree'

/** The node the pointer is currently over, and what dropping there would do. */
export interface FluidDropTarget {
  nodeId: string
  intent: NodeDropIntent
  isAllowed: boolean
}

export interface FluidDragController {
  draggingNodeId: string | null
  /** Set while a *new* block is being dragged out of the picker. */
  draggingBlockId: FluidBlockId | null
  dropTarget: FluidDropTarget | null
  onDragStart: (event: DragEvent<HTMLElement>, nodeId: string) => void
  /** Picks up a catalog entry, so a block can be placed by dropping it. */
  onBlockDragStart: (event: DragEvent<HTMLElement>, blockId: FluidBlockId) => void
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
  /**
   * Preview and commit with an intent the caller worked out itself.
   *
   * The canvas resolves its own geometry — the drop edges there follow the
   * parent's layout axis, not the tree's fixed vertical rows — but must not
   * re-implement validity, which needs the whole tree.
   */
  onDragOverIntent: (
    event: DragEvent<HTMLElement>,
    nodeId: string,
    intent: NodeDropIntent,
  ) => void
  onDropIntent: (event: DragEvent<HTMLElement>, nodeId: string, intent: NodeDropIntent) => void
  onDragLeave: (event: DragEvent<HTMLElement>, nodeId: string) => void
  /**
   * Keyboard parity for reorder, indent, and outdent — all on `Alt` plus an
   * arrow, so `Tab` stays focus navigation.
   */
  onRowKeyDown: (event: KeyboardEvent<HTMLElement>, nodeId: string) => void
  /** Commits a move expressed the same way a drop is. Returns whether it acted. */
  requestMove: (dragId: string, targetId: string | null, intent: NodeDropIntent) => boolean
}

const DROP_OPTIONS = {
  acceptsChildren: canAcceptChildren,
  maxDepth: MAX_NESTING_DEPTH,
}

/**
 * The builder's single drag session, shared by the sections tree and the canvas.
 *
 * Validity lives here rather than in the surfaces because neither a tree row nor
 * a canvas node can answer "would this drop create a cycle?" on its own — that
 * needs the whole tree. Both render whatever `dropTarget` says and stay
 * presentational.
 *
 * One controller for both surfaces is also what makes a drag started in the tree
 * droppable on the canvas, and what stops the two from ever disagreeing about
 * what a drop means.
 */
export function useFluidDrag(
  nodes: ThemeNode[],
  onMoveNode: (nodeId: string, location: NodeLocation) => void,
  onInsertNode?: (node: ThemeNode, location: NodeLocation) => void,
): FluidDragController {
  const [draggingNodeId, setDraggingNodeId] = useState<string | null>(null)
  const [draggingBlockId, setDraggingBlockId] = useState<FluidBlockId | null>(null)
  const draggingBlockRef = useRef<FluidBlockId | null>(null)
  const [dropTarget, setDropTarget] = useState<FluidDropTarget | null>(null)
  // `dataTransfer` payloads are unreadable during dragover, and state updates
  // lag the event stream, so the in-flight drag is mirrored in refs.
  const draggingRef = useRef<string | null>(null)
  const nodesRef = useRef(nodes)
  nodesRef.current = nodes

  const clear = useCallback(() => {
    draggingRef.current = null
    draggingBlockRef.current = null
    setDraggingNodeId(null)
    setDraggingBlockId(null)
    setDropTarget(null)
  }, [])

  /**
   * What the in-flight drag would do at this target.
   *
   * A drag carries either an existing node or a catalog entry, and the two ask
   * different questions: a move can create a cycle or be a no-op, an insertion
   * can do neither. Both share `resolveDestination`, so a drop zone means the
   * same thing whichever is being dragged.
   */
  const resolveForDrag = useCallback((targetId: string | null, intent: NodeDropIntent) => {
    const blockId = draggingBlockRef.current
    if (blockId) {
      const definition = findBlockDefinition(blockId)
      if (!definition) return null
      return resolveInsertion(nodesRef.current, { targetId, intent }, {
        ...DROP_OPTIONS,
        subtreeHeight: subtreeHeight(definition.node as ThemeNode),
      })
    }
    const dragId = draggingRef.current
    if (!dragId) return null
    return resolveDrop(nodesRef.current, { dragId, targetId, intent }, DROP_OPTIONS)
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
      const resolution = resolveForDrag(nodeId, intent)
      if (!resolution) return
      event.preventDefault()
      event.stopPropagation()

      const isAllowed = resolution.ok
      // The drop still has to fire when it is not allowed: the not-allowed
      // cursor shows *that* it is refused, and only the drop can say why.
      event.dataTransfer.dropEffect = isAllowed ? 'move' : 'none'
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
    [resolveForDrag],
  )

  const commitDrop = useCallback(
    (event: DragEvent<HTMLElement>, nodeId: string, intent: NodeDropIntent) => {
      event.preventDefault()
      event.stopPropagation()

      const blockId =
        (event.dataTransfer.getData(BLOCK_DRAG_MIME_TYPE) as FluidBlockId) ||
        draggingBlockRef.current
      const dragId =
        event.dataTransfer.getData(SECTION_DRAG_MIME_TYPE) || draggingRef.current || ''
      const resolution = resolveForDrag(nodeId, intent)
      clear()

      if (!resolution) return
      if (!resolution.ok) {
        // A no-op is a drop that landed where the node already was. Reporting it
        // would be noise, and committing it would add an empty undo entry.
        if (resolution.reason !== 'no-op') toast.error(resolution.message)
        return
      }

      if (blockId) {
        const definition = findBlockDefinition(blockId)
        if (definition) onInsertNode?.(buildFluidNode(definition), resolution.location)
        return
      }
      if (dragId) onMoveNode(dragId, resolution.location)
    },
    [clear, onInsertNode, onMoveNode, resolveForDrag],
  )

  return useMemo(
    () => ({
      draggingNodeId,
      draggingBlockId,
      dropTarget,
      onBlockDragStart: (event, blockId) => {
        event.dataTransfer.setData(BLOCK_DRAG_MIME_TYPE, blockId)
        event.dataTransfer.effectAllowed = 'copy'
        draggingBlockRef.current = blockId
        setDraggingBlockId(blockId)
      },
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
      onDragOverIntent: previewDrop,
      onDropIntent: commitDrop,
      onDragLeave: (event, nodeId) => {
        event.stopPropagation()
        setDropTarget((previous) => (previous?.nodeId === nodeId ? null : previous))
      },
      onRowKeyDown: (event, nodeId) => {
        // One modifier, four directions. `Tab` is deliberately *not* bound:
        // consuming it made the panel a keyboard trap (WCAG 2.1.2), and the
        // earlier compromise — only swallowing it when a move was possible —
        // still trapped focus anywhere in the middle of a tree. Tab is focus
        // navigation again, and structural editing has its own binding.
        if (!event.altKey) return
        const isIndent = event.key === 'ArrowRight'
        const isOutdent = event.key === 'ArrowLeft'
        const isMoveUp = event.key === 'ArrowUp'
        const isMoveDown = event.key === 'ArrowDown'
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
    [clear, commitDrop, draggingBlockId, draggingNodeId, dropTarget, previewDrop, requestMove],
  )
}
