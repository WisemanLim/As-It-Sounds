import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { cn } from "../lib/cn";
import { formatTime } from "../lib/format";
import type { MediaFile } from "../types";

export interface MediaPlayerHandle {
  seek: (ms: number) => void;
  play: () => void;
  pause: () => void;
}

interface Props {
  file: MediaFile;
  src: string;
  onTimeUpdate?: (ms: number) => void;
  onDurationChange?: (ms: number) => void;
}

export const MediaPlayer = forwardRef<MediaPlayerHandle, Props>(function MediaPlayer(
  { file, src, onTimeUpdate, onDurationChange },
  ref,
) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const [_playing, setPlaying] = useState(false);

  const getEl = () => (file.kind === "video" ? videoRef.current : audioRef.current);

  useImperativeHandle(ref, () => ({
    seek: (ms) => {
      const el = getEl();
      if (el) el.currentTime = ms / 1000;
    },
    play: () => getEl()?.play(),
    pause: () => getEl()?.pause(),
  }));

  useEffect(() => {
    setPlaying(false);
    const el = getEl();
    if (el) el.currentTime = 0;
  }, [src, file.kind]);

  const handleTimeUpdate = (e: React.SyntheticEvent<HTMLMediaElement>) => {
    onTimeUpdate?.(Math.floor(e.currentTarget.currentTime * 1000));
  };

  const handleDuration = (e: React.SyntheticEvent<HTMLMediaElement>) => {
    const d = e.currentTarget.duration;
    if (isFinite(d)) onDurationChange?.(Math.floor(d * 1000));
  };

  return (
    <div
      className={cn(
        "overflow-hidden rounded-xl border border-line bg-black",
        file.kind === "audio" && "bg-bg-elevated p-4",
      )}
    >
      {file.kind === "video" ? (
        <video
          ref={videoRef}
          src={src}
          controls
          onTimeUpdate={handleTimeUpdate}
          onLoadedMetadata={handleDuration}
          onPlay={() => setPlaying(true)}
          onPause={() => setPlaying(false)}
          className="aspect-video w-full bg-black"
        />
      ) : (
        <div className="flex flex-col gap-2">
          <div className="truncate text-sm font-medium text-fg">{file.name}</div>
          <audio
            ref={audioRef}
            src={src}
            controls
            onTimeUpdate={handleTimeUpdate}
            onLoadedMetadata={handleDuration}
            onPlay={() => setPlaying(true)}
            onPause={() => setPlaying(false)}
            className="w-full"
          />
          <div className="text-right font-mono text-[11px] text-fg-subtle">
            {formatTime((audioRef.current?.currentTime ?? 0) * 1000)}
          </div>
        </div>
      )}
    </div>
  );
});
