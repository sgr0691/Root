import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const UpdateScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const containerProgress = spring({
    frame: frame - 35,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const narrationProgress = spring({
    frame: frame - 120,
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
            command="root update"
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
            text="Resolving dependencies..."
            delay={50}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Snapshot created"
            delay={70}
            color={theme.colors.text.secondary}
            fontSize={18}
            icon="📸"
          />
          <OutputLine
            text="Updating packages..."
            delay={90}
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
              text="ripgrep 14.1.0 → 14.1.1"
              delay={105}
              color={theme.colors.accent.green}
              fontSize={18}
              icon="✓"
            />
            <OutputLine
              text="Verification complete"
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
          Update everything with deterministic precision.
        </p>
      </div>
    </AbsoluteFill>
  );
};
