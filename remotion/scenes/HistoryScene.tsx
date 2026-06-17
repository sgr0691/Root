import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { CommandRenderer } from "../components/CommandRenderer";
import { Timeline } from "../components/Timeline";

export const HistoryScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const narrationProgress = spring({
    frame: frame - 90,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const events = [
    {
      action: "install",
      package: "terraform",
      timestamp: "2024-01-15 14:32",
      type: "install" as const,
    },
    {
      action: "install",
      package: "kubectl",
      timestamp: "2024-01-15 14:45",
      type: "install" as const,
    },
    {
      action: "rollback",
      package: "jq",
      timestamp: "2024-01-15 15:01",
      type: "rollback" as const,
    },
  ];

  return (
    <AbsoluteFill
      style={{
        backgroundColor: theme.colors.bg.primary,
        justifyContent: "center",
        alignItems: "center",
        padding: 80,
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 36,
          width: "100%",
          maxWidth: 900,
        }}
      >
        <div style={{ marginBottom: 4 }}>
          <CommandRenderer
            command="root history"
            typingDelay={10}
            fontSize={28}
          />
        </div>

        <div
          style={{
            backgroundColor: theme.colors.bg.tertiary,
            borderRadius: theme.radius.lg,
            border: `1px solid ${theme.colors.border.subtle}`,
            padding: 32,
            width: "100%",
          }}
        >
          <Timeline events={events} delay={40} stagger={15} />
        </div>

        <p
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 24,
            color: theme.colors.text.secondary,
            margin: 0,
            textAlign: "center",
            opacity: narrationProgress,
            transform: `translateY(${interpolate(narrationProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          Every change recorded.
        </p>
      </div>
    </AbsoluteFill>
  );
};
