import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";

interface CommandRendererProps {
  command: string;
  showPrompt?: boolean;
  typingDelay?: number;
  typingSpeed?: number;
  fontSize?: number;
}

export const CommandRenderer: React.FC<CommandRendererProps> = ({
  command,
  showPrompt = true,
  typingDelay = 0,
  typingSpeed = 2,
  fontSize = 24,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const elapsed = frame - typingDelay;
  const charCount = Math.floor(elapsed / typingSpeed);
  const visibleText = command.slice(0, Math.max(0, charCount));
  const isTyping = elapsed >= 0 && charCount < command.length;
  const isComplete = charCount >= command.length;

  const opacity = spring({
    frame: frame - typingDelay,
    fps,
    config: { damping: 20, stiffness: 100, mass: 0.5 },
  });

  return (
    <div
      style={{
        fontFamily: theme.fonts.mono,
        fontSize,
        opacity,
        display: "flex",
        alignItems: "center",
      }}
    >
      {showPrompt && (
        <span style={{ color: theme.colors.terminal.prompt, marginRight: 12 }}>
          $
        </span>
      )}
      <span style={{ color: theme.colors.terminal.command }}>
        {visibleText}
      </span>
      {isTyping && (
        <span
          style={{
            display: "inline-block",
            width: 12,
            height: fontSize * 0.9,
            backgroundColor: theme.colors.terminal.command,
            marginLeft: 2,
            verticalAlign: "middle",
          }}
        />
      )}
    </div>
  );
};

interface OutputLineProps {
  text: string;
  delay?: number;
  color?: string;
  fontSize?: number;
  icon?: string;
}

export const OutputLine: React.FC<OutputLineProps> = ({
  text,
  delay = 0,
  color = theme.colors.terminal.output,
  fontSize = 18,
  icon,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 20, stiffness: 100, mass: 0.5 },
  });

  const opacity = progress;
  const translateX = interpolate(progress, [0, 1], [10, 0]);

  return (
    <div
      style={{
        fontFamily: theme.fonts.mono,
        fontSize,
        color,
        opacity,
        transform: `translateX(${translateX}px)`,
        display: "flex",
        alignItems: "center",
        gap: 8,
        marginBottom: 6,
      }}
    >
      {icon && <span>{icon}</span>}
      <span>{text}</span>
    </div>
  );
};
