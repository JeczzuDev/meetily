'use client';

import { memo } from 'react';
import { Trash2, ExternalLink, Bot } from 'lucide-react';
import Markdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import type { SmartNoteResponse } from '@/types/smartNotes';

interface SmartNoteCardProps {
  note: SmartNoteResponse;
  onDelete: (noteId: string) => void;
  isHighlighted?: boolean;
  onHoverNote?: (segmentId: string | null) => void;
}

export const SmartNoteCard = memo(function SmartNoteCard({
  note,
  onDelete,
  isHighlighted,
  onHoverNote,
}: SmartNoteCardProps) {
  const createdAt = new Date(note.created_at).toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });

  return (
    <div
      className={`text-left max-w-full rounded-lg border bg-white p-4 shadow-sm transition-colors duration-150 ${
        isHighlighted ? 'border-blue-200 bg-blue-50 ring-1 ring-blue-200' : 'border-gray-200'
      }`}
      onMouseEnter={onHoverNote ? () => onHoverNote(note.segment_id) : undefined}
      onMouseLeave={onHoverNote ? () => onHoverNote(null) : undefined}
    >
      {/* Content */}
      <div className="prose prose-sm max-w-none text-gray-700 leading-relaxed
        prose-p:my-1 prose-ul:my-1 prose-li:my-0.5 prose-headings:mt-2 prose-headings:mb-1">
        <Markdown remarkPlugins={[remarkGfm]}>{note.content}</Markdown>
      </div>

      {/* Sources */}
      {note.sources && note.sources.length > 0 && (
        <div className="mt-3 border-t border-gray-100 pt-3">
          <p className="text-xs font-medium text-gray-500 mb-1.5">Sources</p>
          <ul className="space-y-1">
            {note.sources.map((source, index) => (
              <li key={index} className="flex items-start gap-1.5">
                <ExternalLink className="w-3 h-3 mt-0.5 flex-shrink-0 text-blue-500" />
                <a
                  href={source.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs text-blue-600 hover:text-blue-800 hover:underline truncate"
                  title={source.snippet}
                >
                  {source.title}
                </a>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Footer: meta + delete */}
      <div className="mt-3 flex items-center justify-between text-xs text-gray-400">
        <div className="flex items-center gap-1.5">
          <Bot className="w-3 h-3" />
          <span>{note.provider}/{note.model}</span>
          <span>&middot;</span>
          <span>{createdAt}</span>
        </div>
        <button
          type="button"
          onClick={() => onDelete(note.id)}
          className="p-1 rounded-md text-gray-400 hover:text-red-500 hover:bg-red-50 transition-colors"
          title="Delete Smart Note"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
});
