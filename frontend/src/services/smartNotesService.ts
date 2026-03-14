/**
 * Smart Notes Service
 *
 * 1-to-1 wrappers for smart_notes Tauri commands.
 * Follows the same singleton pattern as configService.ts and storageService.ts.
 */

import { invoke } from '@tauri-apps/api/core';
import type { SmartNoteRequest, SmartNoteResponse } from '@/types/smartNotes';

export class SmartNotesService {
  /** Generate a Smart Note from transcript context (+ optional web search) */
  async generateSmartNote(request: SmartNoteRequest): Promise<SmartNoteResponse> {
    return invoke<SmartNoteResponse>('generate_smart_note', { request });
  }

  /** Get all Smart Notes for a meeting */
  async getSmartNotes(meetingId: string): Promise<SmartNoteResponse[]> {
    return invoke<SmartNoteResponse[]>('get_smart_notes', { meetingId });
  }

  /** Delete a Smart Note by ID */
  async deleteSmartNote(noteId: string): Promise<{ message: string }> {
    return invoke<{ message: string }>('delete_smart_note', { noteId });
  }
}

export const smartNotesService = new SmartNotesService();
