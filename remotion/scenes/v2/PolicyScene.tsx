import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const PolicyScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const containerProgress = spring({
    frame: frame - 35,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const deniedProgress = spring({
    frame: frame - 110,
    fps,
    config: { damping: 15, stiffness: 100, mass: 0.4 },
  });

  const narrationProgress = spring({
    frame: frame - 140,
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
          command="root policy apply policy.toml"
          typingDelay={10}
          fontSize={28}
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
            text="Policy activated"
            delay={50}
            color={theme.colors.accent.green}
            fontSize={18}
            icon="✓"
          />
          <OutputLine
            text="4 rules loaded"
            delay={65}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
        </div>

        <div style={{ width: "100%", maxWidth: 900 }}>
          <CommandRenderer
            command="root remove ripgrep"
            typingDelay={75}
            fontSize={24}
          />
        </div>

        <div
          style={{
            backgroundColor: `${theme.colors.accent.red}08`,
            borderRadius: theme.radius.lg,
            border: `1.5px solid ${theme.colors.accent.red}40`,
            padding: 24,
            width: "100%",
            opacity: deniedProgress,
            transform: `translateY(${interpolate(deniedProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              fontFamily: theme.fonts.mono,
              fontSize: 20,
              fontWeight: 700,
              color: theme.colors.accent.red,
            }}
          >
            <span style={{ fontSize: 24 }}>✗</span>
            DENIED
          </div>
          <OutputLine
            text="Policy violation: package removal not permitted"
            delay={115}
            color={theme.colors.accent.red}
            fontSize={16}
          />
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
          Set guardrails. Never break your rules.
        </p>
      </div>
    </AbsoluteFill>
  );
};
