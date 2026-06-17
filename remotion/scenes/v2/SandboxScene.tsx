import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const SandboxScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const containerProgress = spring({
    frame: frame - 50,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const containerProgress2 = spring({
    frame: frame - 130,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const narrationProgress = spring({
    frame: frame - 140,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const narrationProgress2 = spring({
    frame: frame - 160,
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
          gap: 32,
          width: "100%",
          maxWidth: 900,
        }}
      >
        <CommandRenderer
          command="root sandbox create test-env --image ubuntu:latest"
          typingDelay={10}
          typingSpeed={1.5}
          fontSize={24}
        />

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
            text="Creating sandbox..."
            delay={60}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Pulling ubuntu:latest..."
            delay={80}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Sandbox ready"
            delay={100}
            color={theme.colors.accent.green}
            fontSize={18}
            icon="✓"
          />
        </div>

        <div style={{ width: "100%", maxWidth: 900 }}>
          <CommandRenderer
            command="root sandbox run test-env -- echo isolated"
            typingDelay={110}
            typingSpeed={1.5}
            fontSize={24}
          />
        </div>

        <div
          style={{
            backgroundColor: theme.colors.bg.tertiary,
            borderRadius: theme.radius.lg,
            border: `1px solid ${theme.colors.border.subtle}`,
            padding: 24,
            width: "100%",
            opacity: containerProgress2,
            transform: `translateY(${interpolate(containerProgress2, [0, 1], [10, 0])}px)`,
          }}
        >
          <OutputLine
            text="isolated"
            delay={140}
            color={theme.colors.text.primary}
            fontSize={18}
          />
          <OutputLine
            text="Sandbox destroyed"
            delay={155}
            color={theme.colors.accent.green}
            fontSize={18}
            icon="✓"
          />
        </div>

        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 4 }}>
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
            Experiment in isolation.
          </p>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 18,
              color: theme.colors.text.tertiary,
              margin: 0,
              textAlign: "center",
              opacity: narrationProgress2,
              transform: `translateY(${interpolate(narrationProgress2, [0, 1], [10, 0])}px)`,
            }}
          >
            Zero risk to your machine.
          </p>
        </div>
      </div>
    </AbsoluteFill>
  );
};
