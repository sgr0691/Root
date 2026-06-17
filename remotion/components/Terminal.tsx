import React from "react";
import { useCurrentFrame, useVideoConfig, spring } from "remotion";
import { theme } from "../styles/theme";

export interface TerminalLine {
  text: string;
  type: "command" | "output" | "success" | "error" | "prompt";
  delay?: number;
}

interface TerminalProps {
  lines: TerminalLine[];
  width?: number;
  height?: number;
  typingSpeed?: number;
  showCursor?: boolean;
}

export const Terminal: React.FC<TerminalProps> = ({
  lines,
  width = 900,
  height = 500,
  typingSpeed = 2,
  showCursor = true,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const getVisibleText = (line: TerminalLine, lineIndex: number) => {
    const lineStartFrame = line.delay ?? lineIndex * 20;
    const elapsed = frame - lineStartFrame;

    if (elapsed < 0) return "";

    if (line.type === "command") {
      const charCount = Math.floor(elapsed / typingSpeed);
      return line.text.slice(0, charCount);
    }

    return line.text;
  };

  const isLineVisible = (line: TerminalLine, lineIndex: number) => {
    const lineStartFrame = line.delay ?? lineIndex * 20;
    return frame >= lineStartFrame;
  };

  const isTypingComplete = (line: TerminalLine, lineIndex: number) => {
    if (line.type !== "command") return true;
    const lineStartFrame = line.delay ?? lineIndex * 20;
    const elapsed = frame - lineStartFrame;
    return elapsed >= line.text.length * typingSpeed;
  };

  const isCurrentLine = (lineIndex: number) => {
    const line = lines[lineIndex];
    const lineStartFrame = line.delay ?? lineIndex * 20;
    const nextLineStart = lines[lineIndex + 1]?.delay ?? (lineIndex + 1) * 20;
    return frame >= lineStartFrame && frame < nextLineStart;
  };

  const getLineColor = (type: TerminalLine["type"]) => {
    switch (type) {
      case "command":
        return theme.colors.terminal.command;
      case "success":
        return theme.colors.terminal.success;
      case "error":
        return theme.colors.terminal.error;
      case "prompt":
        return theme.colors.terminal.prompt;
      default:
        return theme.colors.terminal.output;
    }
  };

  return (
    <div
      style={{
        width,
        height,
        backgroundColor: theme.colors.terminal.bg,
        borderRadius: theme.radius.lg,
        border: `1px solid ${theme.colors.terminal.border}`,
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.5)",
      }}
    >
      <div
        style={{
          height: 40,
          backgroundColor: theme.colors.bg.tertiary,
          borderBottom: `1px solid ${theme.colors.terminal.border}`,
          display: "flex",
          alignItems: "center",
          padding: `0 ${theme.spacing.md}px`,
          gap: 8,
        }}
      >
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            backgroundColor: "#ef4444",
          }}
        />
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            backgroundColor: "#f59e0b",
          }}
        />
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            backgroundColor: "#22c55e",
          }}
        />
        <span
          style={{
            marginLeft: theme.spacing.md,
            fontSize: 13,
            color: theme.colors.text.tertiary,
            fontFamily: theme.fonts.mono,
          }}
        >
          terminal
        </span>
      </div>

      <div
        style={{
          flex: 1,
          padding: theme.spacing.lg,
          fontFamily: theme.fonts.mono,
          fontSize: 18,
          lineHeight: 1.6,
          overflow: "hidden",
        }}
      >
        {lines.map((line, i) => {
          if (!isLineVisible(line, i)) return null;

          const visibleText = getVisibleText(line, i);
          const typingDone = isTypingComplete(line, i);
          const isCurrent = isCurrentLine(i) && line.type === "command" && !typingDone;

          return (
            <div key={i} style={{ marginBottom: 4 }}>
              {line.type === "prompt" && (
                <span style={{ color: theme.colors.terminal.prompt }}>
                  ${" "}
                </span>
              )}
              <span style={{ color: getLineColor(line.type) }}>
                {visibleText}
              </span>
              {isCurrent && showCursor && (
                <span
                  style={{
                    display: "inline-block",
                    width: 10,
                    height: 20,
                    backgroundColor: theme.colors.terminal.command,
                    marginLeft: 2,
                    verticalAlign: "middle",
                    animation: "blink 1s step-end infinite",
                  }}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
