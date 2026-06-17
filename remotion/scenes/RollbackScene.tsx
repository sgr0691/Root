import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { CommandRenderer, OutputLine } from "../components/CommandRenderer";
import { FadeIn } from "../components/FadeIn";

export const RollbackScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const outputStart = 50;

  const successProgress = spring({
    frame: frame - outputStart - 55,
    fps,
    config: { damping: 12, stiffness: 100, mass: 0.4 },
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
            command="root rollback"
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
            text="Rolling back..."
            delay={outputStart}
            color={theme.colors.text.secondary}
            fontSize={18}
          />
          <OutputLine
            text="Restoring previous state..."
            delay={outputStart + 20}
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
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                opacity: successProgress,
                transform: `scale(${interpolate(successProgress, [0, 1], [0.9, 1])})`,
              }}
            >
              <div
                style={{
                  width: 36,
                  height: 36,
                  borderRadius: "50%",
                  backgroundColor: `${theme.colors.accent.green}20`,
                  border: `2px solid ${theme.colors.accent.green}`,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: 18,
                  color: theme.colors.accent.green,
                  fontWeight: 700,
                }}
              >
                ✓
              </div>
              <span
                style={{
                  fontFamily: theme.fonts.sans,
                  fontSize: 22,
                  fontWeight: 600,
                  color: theme.colors.accent.green,
                }}
              >
                Rollback completed
              </span>
            </div>

            <FadeIn delay={outputStart + 75}>
              <OutputLine
                text="State restored"
                delay={0}
                color={theme.colors.accent.green}
                fontSize={20}
                icon="✓"
              />
            </FadeIn>
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
          Safe to experiment.
        </p>
      </div>
    </AbsoluteFill>
  );
};
