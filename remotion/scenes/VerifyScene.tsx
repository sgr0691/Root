import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { CommandRenderer, OutputLine } from "../components/CommandRenderer";

export const VerifyScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const outputStart = 50;

  const checkProgress = spring({
    frame: frame - outputStart - 50,
    fps,
    config: { damping: 12, stiffness: 100, mass: 0.4 },
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
            command="root verify terraform"
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
            text="Running: terraform version"
            delay={outputStart}
            color={theme.colors.text.secondary}
            fontSize={18}
          />

          <div style={{ marginTop: 16 }}>
            <OutputLine
              text="Terraform v1.5.0"
              delay={outputStart + 25}
              color={theme.colors.text.primary}
              fontSize={18}
            />
          </div>

          <div
            style={{
              marginTop: 24,
              paddingTop: 24,
              borderTop: `1px solid ${theme.colors.border.subtle}`,
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                opacity: checkProgress,
                transform: `scale(${interpolate(checkProgress, [0, 1], [0.9, 1])})`,
              }}
            >
              <div
                style={{
                  width: 40,
                  height: 40,
                  borderRadius: "50%",
                  backgroundColor: `${theme.colors.accent.green}20`,
                  border: `2px solid ${theme.colors.accent.green}`,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: 20,
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
                verified
              </span>
            </div>
          </div>
        </div>

        <div style={{ textAlign: "center" }}>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 24,
              color: theme.colors.text.primary,
              margin: 0,
              opacity: narrationProgress,
              transform: `translateY(${interpolate(narrationProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            Don&apos;t assume.
          </p>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 24,
              color: theme.colors.text.secondary,
              margin: "4px 0 0 0",
              opacity: narrationProgress,
              transform: `translateY(${interpolate(narrationProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            Verify.
          </p>
        </div>
      </div>
    </AbsoluteFill>
  );
};
