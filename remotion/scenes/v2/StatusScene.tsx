import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const StatusScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const containerProgress = spring({
    frame: frame - 35,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const statusProgress = spring({
    frame: frame - 95,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const narrationProgress = spring({
    frame: frame - 100,
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
            command="root status"
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
            text="Machine ID: sergios-macbook"
            delay={50}
            color={theme.colors.text.secondary}
            fontSize={16}
          />

          <div style={{ height: 1, backgroundColor: theme.colors.border.subtle, margin: "16px 0" }} />

          <OutputLine
            text="Rootfile"
            delay={60}
            color={theme.colors.text.muted}
            fontSize={12}
          />
          <OutputLine
            text="OK"
            delay={65}
            color={theme.colors.accent.green}
            fontSize={18}
            icon="✓"
          />

          <OutputLine
            text="Lockfile"
            delay={72}
            color={theme.colors.text.muted}
            fontSize={12}
          />
          <OutputLine
            text="OK"
            delay={77}
            color={theme.colors.accent.green}
            fontSize={18}
            icon="✓"
          />

          <OutputLine
            text="Profile"
            delay={84}
            color={theme.colors.text.muted}
            fontSize={12}
          />
          <OutputLine
            text="OK"
            delay={89}
            color={theme.colors.accent.green}
            fontSize={18}
            icon="✓"
          />

          <div
            style={{
              marginTop: 24,
              paddingTop: 24,
              borderTop: `1px solid ${theme.colors.border.subtle}`,
              display: "flex",
              alignItems: "center",
              gap: 12,
              opacity: statusProgress,
              transform: `translateY(${interpolate(statusProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            <span
              style={{
                fontFamily: theme.fonts.mono,
                fontSize: 11,
                color: theme.colors.text.muted,
                textTransform: "uppercase",
                letterSpacing: "0.1em",
              }}
            >
              Status
            </span>
            <span
              style={{
                fontFamily: theme.fonts.sans,
                fontSize: 22,
                fontWeight: 700,
                color: theme.colors.accent.green,
              }}
            >
              Healthy
            </span>
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
          Know your machine state at a glance.
        </p>
      </div>
    </AbsoluteFill>
  );
};
