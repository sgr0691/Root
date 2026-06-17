import React from "react";
import { OffthreadVideo } from "remotion";
import { Terminal, TerminalLine } from "./Terminal";
import { theme } from "../styles/theme";

export interface TerminalPlaybackProps {
  lines: TerminalLine[];
  width?: number;
  height?: number;
  typingSpeed?: number;
  showCursor?: boolean;
  source?: "synthetic" | "recording";
  recordingSrc?: string;
  startFrom?: number;
  endAt?: number;
}

export const TerminalPlayback: React.FC<TerminalPlaybackProps> = ({
  lines,
  width = 900,
  height = 500,
  typingSpeed = 2,
  showCursor = true,
  source = "synthetic",
  recordingSrc,
  startFrom = 0,
  endAt,
}) => {
  if (source === "recording" && recordingSrc) {
    return (
      <div
        style={{
          width,
          height,
          borderRadius: theme.radius.lg,
          overflow: "hidden",
          border: `1px solid ${theme.colors.terminal.border}`,
          boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.5)",
        }}
      >
        <OffthreadVideo
          src={recordingSrc}
          style={{ width: "100%", height: "100%", objectFit: "cover" }}
          startFrom={startFrom}
          endAt={endAt}
        />
      </div>
    );
  }

  return (
    <Terminal
      lines={lines}
      width={width}
      height={height}
      typingSpeed={typingSpeed}
      showCursor={showCursor}
    />
  );
};
