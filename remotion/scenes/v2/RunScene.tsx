import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const RunScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const containerProgress = spring({
    frame: frame - 35,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const narrationProgress = spring({
    frame: frame - 130,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

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
            command="root run dev"
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
            opacity: containerProgress,
            transform: `translateY(${interpolate(containerProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          <OutputLine
            text="Executing task: dev"
            delay={50}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Environment: Root-managed profile"
            delay={65}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Running: cargo build && cargo run"
            delay={80}
            color={theme.colors.text.secondary}
            fontSize={18}
          />

          <div
            style={{
              marginTop: 24,
              paddingTop: 24,
              borderTop: `1px solid ${theme.colors.border.subtle}`,
              display: "flex",
              flexDirection: "column",
              gap: 10,
            }}
          >
            <OutputLine
              text="Build successful"
              delay={100}
              color={theme.colors.accent.green}
              fontSize={18}
              icon="✓"
            />
            <OutputLine
              text="Task completed"
              delay={115}
              color={theme.colors.accent.green}
              fontSize={18}
              icon="✓"
            />
          </div>
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
          Run tasks with the right environment every time.
        </p>
      </div>
    </AbsoluteFill>
  );
};
