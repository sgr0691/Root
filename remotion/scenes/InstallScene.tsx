import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { CommandRenderer, OutputLine } from "../components/CommandRenderer";
import { BadgeGroup } from "../components/Badge";

export const InstallScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const outputStart = 50;

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
            command="root install terraform"
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
          <OutputLine
            text="Installing terraform..."
            delay={outputStart}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Resolving dependencies..."
            delay={outputStart + 20}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Downloading terraform 1.5.0..."
            delay={outputStart + 40}
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
              text="Installed"
              delay={outputStart + 65}
              color={theme.colors.accent.green}
              fontSize={20}
              icon="✓"
            />
            <OutputLine
              text="Verified"
              delay={outputStart + 80}
              color={theme.colors.accent.green}
              fontSize={20}
              icon="✓"
            />
            <OutputLine
              text="Rollback Available"
              delay={outputStart + 95}
              color={theme.colors.accent.green}
              fontSize={20}
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
          Install with verification built in.
        </p>
      </div>
    </AbsoluteFill>
  );
};
