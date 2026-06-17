import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const SearchScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const containerProgress = spring({
    frame: frame - 35,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const narrationProgress = spring({
    frame: frame - 90,
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
            command="root search ripgrep"
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
            text="ripgrep"
            delay={50}
            color={theme.colors.text.primary}
            fontSize={22}
            icon="📦"
          />

          <div style={{ height: 1, backgroundColor: theme.colors.border.subtle, margin: "12px 0" }} />

          <OutputLine
            text="Description"
            delay={60}
            color={theme.colors.text.muted}
            fontSize={12}
          />
          <OutputLine
            text="A line-oriented search tool that recursively searches directories"
            delay={65}
            color={theme.colors.text.secondary}
            fontSize={16}
          />

          <div style={{ height: 1, backgroundColor: theme.colors.border.subtle, margin: "12px 0" }} />

          <div style={{ display: "flex", gap: 40 }}>
            <div>
              <OutputLine
                text="Binary"
                delay={75}
                color={theme.colors.text.muted}
                fontSize={12}
              />
              <OutputLine
                text="rg"
                delay={80}
                color={theme.colors.accent.blue}
                fontSize={16}
              />
            </div>
            <div>
              <OutputLine
                text="Version"
                delay={75}
                color={theme.colors.text.muted}
                fontSize={12}
              />
              <OutputLine
                text="14.1.0"
                delay={80}
                color={theme.colors.accent.blue}
                fontSize={16}
              />
            </div>
            <div>
              <OutputLine
                text="Category"
                delay={75}
                color={theme.colors.text.muted}
                fontSize={12}
              />
              <OutputLine
                text="terminal"
                delay={80}
                color={theme.colors.accent.blue}
                fontSize={16}
              />
            </div>
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
          Find supported tools instantly.
        </p>
      </div>
    </AbsoluteFill>
  );
};
