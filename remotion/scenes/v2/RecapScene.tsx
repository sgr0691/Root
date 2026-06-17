import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { FadeIn } from "../../components/FadeIn";
import { Logo } from "../../components/Logo";

export const RecapScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const steps = ["Plan", "Install", "Verify", "Rollback", "History"];

  const transitionProgress = spring({
    frame: frame - 55,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const versionProgress = spring({
    frame: frame - 70,
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
          gap: 28,
          width: "100%",
          maxWidth: 900,
        }}
      >
        <FadeIn delay={0}>
          <Logo size="medium" />
        </FadeIn>

        <div
          style={{
            display: "flex",
            gap: 16,
            flexWrap: "wrap",
            justifyContent: "center",
          }}
        >
          {steps.map((step, i) => (
            <FadeIn key={i} delay={5 + i * 10}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  fontFamily: theme.fonts.sans,
                  fontSize: 18,
                  fontWeight: 600,
                  color: theme.colors.text.primary,
                  padding: "10px 20px",
                  borderRadius: theme.radius.md,
                  backgroundColor: theme.colors.bg.tertiary,
                  border: `1px solid ${theme.colors.border.subtle}`,
                }}
              >
                <span style={{ color: theme.colors.accent.green }}>✓</span>
                {step}
              </div>
            </FadeIn>
          ))}
        </div>

        <FadeIn delay={55}>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 24,
              color: theme.colors.text.secondary,
              margin: 0,
              textAlign: "center",
              opacity: transitionProgress,
              transform: `translateY(${interpolate(transitionProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            Root explained every install.
          </p>
        </FadeIn>

        <FadeIn delay={68}>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 8,
              opacity: versionProgress,
              transform: `translateY(${interpolate(versionProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            <p
              style={{
                fontFamily: theme.fonts.sans,
                fontSize: 28,
                fontWeight: 600,
                color: theme.colors.text.primary,
                margin: 0,
              }}
            >
              Now it manages your environment.
            </p>
            <span
              style={{
                fontFamily: theme.fonts.mono,
                fontSize: 16,
                color: theme.colors.accent.blue,
              }}
            >
              Root v0.2
            </span>
          </div>
        </FadeIn>
      </div>
    </AbsoluteFill>
  );
};
