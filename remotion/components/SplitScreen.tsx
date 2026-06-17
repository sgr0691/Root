import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";

interface SplitScreenProps {
  left: React.ReactNode;
  right: React.ReactNode;
  leftDelay?: number;
  rightDelay?: number;
  gap?: number;
}

export const SplitScreen: React.FC<SplitScreenProps> = ({
  left,
  right,
  leftDelay = 0,
  rightDelay = 15,
  gap = 24,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const leftProgress = spring({
    frame: frame - leftDelay,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const rightProgress = spring({
    frame: frame - rightDelay,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  return (
    <div
      style={{
        display: "flex",
        gap,
        width: "100%",
        maxWidth: 1100,
      }}
    >
      <div
        style={{
          flex: 1,
          opacity: leftProgress,
          transform: `translateX(${interpolate(leftProgress, [0, 1], [-20, 0])}px)`,
        }}
      >
        {left}
      </div>
      <div
        style={{
          flex: 1,
          opacity: rightProgress,
          transform: `translateX(${interpolate(rightProgress, [0, 1], [20, 0])}px)`,
        }}
      >
        {right}
      </div>
    </div>
  );
};

interface SplitTerminalProps {
  title: string;
  children: React.ReactNode;
}

export const SplitTerminal: React.FC<SplitTerminalProps> = ({
  title,
  children,
}) => {
  return (
    <div
      style={{
        backgroundColor: theme.colors.bg.tertiary,
        borderRadius: theme.radius.lg,
        border: `1px solid ${theme.colors.border.subtle}`,
        padding: 24,
        display: "flex",
        flexDirection: "column",
        gap: 16,
        height: "100%",
      }}
    >
      <div
        style={{
          fontFamily: theme.fonts.mono,
          fontSize: 11,
          color: theme.colors.text.muted,
          textTransform: "uppercase",
          letterSpacing: "0.1em",
        }}
      >
        {title}
      </div>
      {children}
    </div>
  );
};
