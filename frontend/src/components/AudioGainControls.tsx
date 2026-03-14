'use client';

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Mic, Speaker, Volume2 } from 'lucide-react';
import { Label } from '@/components/ui/label';

interface AudioGainControlsProps {
  disabled?: boolean;
  className?: string;
}

const STORAGE_KEY_MIC = 'audioGain_mic';
const STORAGE_KEY_SYSTEM = 'audioGain_system';

export function AudioGainControls({ disabled = false, className = '' }: AudioGainControlsProps) {
  const [micGain, setMicGain] = useState(1.0);
  const [systemGain, setSystemGain] = useState(1.0);
  const syncedRef = useRef(false);

  // Load saved gains from localStorage on mount and sync to Rust
  useEffect(() => {
    if (syncedRef.current) return;
    syncedRef.current = true;

    const savedMic = localStorage.getItem(STORAGE_KEY_MIC);
    const savedSys = localStorage.getItem(STORAGE_KEY_SYSTEM);

    const mic = savedMic ? parseFloat(savedMic) : 1.0;
    const sys = savedSys ? parseFloat(savedSys) : 1.0;

    setMicGain(mic);
    setSystemGain(sys);

    invoke('set_audio_gains', { micGain: mic, systemGain: sys }).catch(err =>
      console.error('Failed to sync audio gains to Rust on startup:', err)
    );
  }, []);

  const syncGains = useCallback((mic: number, sys: number) => {
    localStorage.setItem(STORAGE_KEY_MIC, mic.toString());
    localStorage.setItem(STORAGE_KEY_SYSTEM, sys.toString());
    invoke('set_audio_gains', { micGain: mic, systemGain: sys }).catch(err =>
      console.error('Failed to set audio gains:', err)
    );
  }, []);

  const handleMicChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const val = parseFloat(e.target.value);
    setMicGain(val);
    syncGains(val, systemGain);
  }, [systemGain, syncGains]);

  const handleSystemChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const val = parseFloat(e.target.value);
    setSystemGain(val);
    syncGains(micGain, val);
  }, [micGain, syncGains]);

  const formatPercent = (val: number) => `${Math.round(val * 100)}%`;

  return (
    <div className={`space-y-4 ${className}`}>
      <div className="flex items-center gap-2">
        <Volume2 className="h-4 w-4 text-gray-600" />
        <h4 className="text-sm font-medium text-gray-900">Audio Mix Balance</h4>
      </div>

      {/* Mic Gain */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <Mic className="h-3.5 w-3.5 text-gray-500" />
            <Label className="text-xs text-gray-600">Microphone</Label>
          </div>
          <span className="text-xs font-mono text-gray-500 min-w-[3rem] text-right">
            {formatPercent(micGain)}
          </span>
        </div>
        <input
          type="range"
          min="0"
          max="2"
          step="0.05"
          value={micGain}
          onChange={handleMicChange}
          disabled={disabled}
          className="w-full h-1.5 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        />
      </div>

      {/* System Audio Gain */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <Speaker className="h-3.5 w-3.5 text-gray-500" />
            <Label className="text-xs text-gray-600">System Audio</Label>
          </div>
          <span className="text-xs font-mono text-gray-500 min-w-[3rem] text-right">
            {formatPercent(systemGain)}
          </span>
        </div>
        <input
          type="range"
          min="0"
          max="3"
          step="0.05"
          value={systemGain}
          onChange={handleSystemChange}
          disabled={disabled}
          className="w-full h-1.5 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        />
      </div>

      <p className="text-xs text-gray-500">
        Adjust the volume balance between your microphone and system audio capture. Increase system audio if computer sounds are too quiet in the recording.
      </p>
    </div>
  );
}
