'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { Sparkles, X, Loader2, Search, GripVertical } from 'lucide-react';
import { AnimatePresence, motion } from 'framer-motion';
import { SmartNoteCard } from './SmartNoteCard';
import type { SmartNoteResponse, SmartNoteStatus, ContextSegment } from '@/types/smartNotes';
import type { TranscriptSegmentData } from '@/types';

const MIN_WIDTH = 280;
const MAX_WIDTH = 600;
const DEFAULT_WIDTH = 360;

interface SmartNotesPanelProps {
  /** Whether the panel is visible */
  isOpen: boolean;
  onClose: () => void;
  /** Notes returned by useSmartNotes */
  notes: SmartNoteResponse[];
  status: SmartNoteStatus;
  error: string | null;
  activeSegmentId: string | null;
  /** Callbacks from useSmartNotes */
  onDeleteNote: (noteId: string) => void;
  onLoadNotes: () => void;
  /** All transcript segments (used to build context window) */
  segments: TranscriptSegmentData[];
  /** The hook's generateNote wrapped by parent for convenience */
  onGenerateNote: (segmentId: string, segmentText: string) => void;
  /** Web-search toggle state */
  useWebSearch: boolean;
  onToggleWebSearch: (value: boolean) => void;
  /** When true, panel floats over content instead of pushing it */
  overlay?: boolean;
  /** Segment ID being hovered (for cross-highlighting with transcript) */
  hoveredNoteSegmentId?: string | null;
  /** Called when user hovers a SmartNoteCard */
  onHoverNote?: (segmentId: string | null) => void;
}

export function SmartNotesPanel({
  isOpen,
  onClose,
  notes,
  status,
  error,
  activeSegmentId,
  onDeleteNote,
  onLoadNotes,
  segments,
  onGenerateNote,
  useWebSearch,
  onToggleWebSearch,
  overlay = false,
  hoveredNoteSegmentId,
  onHoverNote,
}: SmartNotesPanelProps) {
  const [width, setWidth] = useState(DEFAULT_WIDTH);
  const isDragging = useRef(false);
  const startX = useRef(0);
  const startWidth = useRef(DEFAULT_WIDTH);

  // Load existing notes when panel first opens
  useEffect(() => {
    if (isOpen) {
      onLoadNotes();
    }
  }, [isOpen, onLoadNotes]);

  // Resize handlers
  const handleDragStart = useCallback((e: React.PointerEvent) => {
    isDragging.current = true;
    startX.current = e.clientX;
    startWidth.current = width;
    e.currentTarget.setPointerCapture(e.pointerId);
  }, [width]);

  const handleDragMove = useCallback((e: React.PointerEvent) => {
    if (!isDragging.current) return;
    const delta = startX.current - e.clientX; // dragging left = wider
    const newWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth.current + delta));
    setWidth(newWidth);
  }, []);

  const handleDragEnd = useCallback(() => {
    isDragging.current = false;
  }, []);

  const isGenerating = status === 'generating';

  const panelContent = (
    <>
      {/* Resize handle (left edge) */}
      <div
        onPointerDown={handleDragStart}
        onPointerMove={handleDragMove}
        onPointerUp={handleDragEnd}
        className="absolute left-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-400/40 active:bg-blue-500/50 transition-colors z-10"
      />

      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-blue-600" />
          <h3 className="text-sm font-semibold text-gray-900">Smart Notes</h3>
          {notes.length > 0 && (
            <span className="text-xs bg-blue-100 text-blue-700 rounded-full px-2 py-0.5">
              {notes.length}
            </span>
          )}
        </div>
        <button
          type="button"
          onClick={onClose}
          className="p-1 rounded-md text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Web search toggle */}
      <div className="px-4 py-2 border-b border-gray-100 flex items-center justify-between">
        <label className="flex items-center gap-2 text-xs text-gray-600 cursor-pointer select-none">
          <Search className="w-3.5 h-3.5" />
          Web search
        </label>
        <button
          type="button"
          onClick={() => onToggleWebSearch(!useWebSearch)}
          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
            useWebSearch ? 'bg-blue-600' : 'bg-gray-300'
          }`}
        >
          <span
            className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${
              useWebSearch ? 'translate-x-[18px]' : 'translate-x-0.5'
            }`}
          />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-4 py-3 space-y-3">
        {/* Generating indicator */}
        {isGenerating && (
          <div className="flex items-center gap-2 text-sm text-blue-600 bg-blue-50 rounded-lg px-3 py-2">
            <Loader2 className="w-4 h-4 animate-spin" />
            <span>Generating note...</span>
          </div>
        )}

        {/* Error */}
        {error && (
          <div className="text-sm text-red-600 bg-red-50 rounded-lg px-3 py-2">
            {error}
          </div>
        )}

        {/* Notes list */}
        {notes.map((note) => (
          <SmartNoteCard
            key={note.id}
            note={note}
            onDelete={onDeleteNote}
            isHighlighted={hoveredNoteSegmentId === note.segment_id}
            onHoverNote={onHoverNote}
          />
        ))}

        {/* Empty state */}
        {notes.length === 0 && !isGenerating && !error && (
          <div className="text-center py-8 text-gray-400">
            <Sparkles className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm font-medium">No Smart Notes yet</p>
            <p className="text-xs mt-1">
              Hover over a transcript segment and click the ✨ icon to generate a note.
            </p>
          </div>
        )}
      </div>
    </>
  );

  if (overlay) {
    return (
      <AnimatePresence>
        {isOpen && (
          <>
            {/* Backdrop */}
            {/* <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="fixed inset-0 bg-black/20 z-40"
              onClick={onClose}
            /> */}
            {/* Panel */}
            <motion.div
              initial={{ x: '100%' }}
              animate={{ x: 0 }}
              exit={{ x: '100%' }}
              transition={{ duration: 0.2, ease: 'easeInOut' }}
              style={{ width }}
              className="fixed top-0 right-0 bottom-0 z-50 border-l border-gray-200 bg-white flex flex-col relative"
            >
              {panelContent}
            </motion.div>
          </>
        )}
      </AnimatePresence>
    );
  }

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ width: 0, opacity: 0 }}
          animate={{ width, opacity: 1 }}
          exit={{ width: 0, opacity: 0 }}
          transition={{ duration: 0.2, ease: 'easeInOut' }}
          style={{ width }}
          className="flex-shrink-0 border-l border-gray-200 bg-white flex flex-col overflow-hidden relative"
        >
          {panelContent}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
