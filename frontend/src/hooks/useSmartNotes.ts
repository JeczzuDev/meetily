import { useState, useCallback, useRef, useMemo, useEffect } from 'react';
import { smartNotesService } from '@/services/smartNotesService';
import type {
  SmartNoteResponse,
  SmartNoteStatus,
  ContextSegment,
} from '@/types/smartNotes';
import { toast } from 'sonner';

interface UseSmartNotesProps {
  meetingId: string;
}

export function useSmartNotes({ meetingId }: UseSmartNotesProps) {
  const [notes, setNotes] = useState<SmartNoteResponse[]>([]);
  const [status, setStatus] = useState<SmartNoteStatus>('idle');
  const [error, setError] = useState<string | null>(null);
  const [activeSegmentId, setActiveSegmentId] = useState<string | null>(null);

  // Track the current generation to allow cancellation via a new request
  const generationRef = useRef(0);

  /** Load all existing notes for this meeting */
  const loadNotes = useCallback(async () => {
    try {
      const result = await smartNotesService.getSmartNotes(meetingId);
      setNotes(result);
    } catch (err) {
      console.error('Failed to load smart notes:', err);
    }
  }, [meetingId]);

  // Auto-load notes when meetingId changes (fixes persistence across navigation)
  useEffect(() => {
    if (meetingId) {
      loadNotes();
    }
  }, [meetingId, loadNotes]);

  /** Generate a Smart Note for a specific transcript segment */
  const generateNote = useCallback(
    async (
      segmentId: string,
      segmentText: string,
      contextSegments: ContextSegment[],
      useWebSearch: boolean = false,
      language?: string,
    ) => {
      // Bump generation id — any previous in-flight request becomes stale
      const currentGeneration = ++generationRef.current;

      setStatus('generating');
      setError(null);
      setActiveSegmentId(segmentId);

      try {
        const note = await smartNotesService.generateSmartNote({
          meeting_id: meetingId,
          segment_id: segmentId,
          segment_text: segmentText,
          context_segments: contextSegments,
          use_web_search: useWebSearch,
          language: language && language !== 'auto' ? language : undefined,
        });

        // Ignore response if a newer generation was started
        if (currentGeneration !== generationRef.current) return;

        setNotes((prev) => [note, ...prev]);
        setStatus('idle');
        toast.success('Smart Note generated');
      } catch (err) {
        if (currentGeneration !== generationRef.current) return;

        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        setStatus('error');
        toast.error('Failed to generate Smart Note', { description: message });
      }
    },
    [meetingId],
  );

  /** Delete a Smart Note */
  const deleteNote = useCallback(async (noteId: string) => {
    try {
      await smartNotesService.deleteSmartNote(noteId);
      setNotes((prev) => prev.filter((n) => n.id !== noteId));
      toast.success('Smart Note deleted');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error('Failed to delete Smart Note', { description: message });
    }
  }, []);

  /** Reset state (useful when switching meetings) */
  const reset = useCallback(() => {
    generationRef.current++;
    setNotes([]);
    setStatus('idle');
    setError(null);
    setActiveSegmentId(null);
  }, []);

  /** Set of segment IDs that already have a Smart Note */
  const processedSegmentIds = useMemo(
    () => new Set(notes.map((n) => n.segment_id)),
    [notes],
  );

  return {
    notes,
    status,
    error,
    activeSegmentId,
    isGenerating: status === 'generating',
    processedSegmentIds,
    loadNotes,
    generateNote,
    deleteNote,
    reset,
  };
}
