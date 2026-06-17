import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../../styles/theme";
import { FadeIn } from "../../../components/FadeIn";
import { Logo } from "../../../components/Logo";

export const RecapShort: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const steps = ["Plan", "Install", "Verify", "Rollback"];

  const transitionProgress = spring({
    frame: frame - 35,
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
          gap: 24,
          width: "100%",
          maxWidth: 800,
        }}
      >
        <FadeIn delay={0}>
          <Logo size="small" />
        </FadeIn>

        <div
          style={{
            display: "flex",
            gap: 14,
            flexWrap: "wrap",
            justifyContent: "center",
          }}
        >
          {steps.map((step, i) => (
            <FadeIn key={i} delay={3 + i * 8}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  fontFamily: theme.fonts.sans,
                  fontSize: 16,
                  fontWeight: 600,
                  color: theme.colors.text.primary,
                  padding: "8px 16px",
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

        <FadeIn delay={35}>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 22,
              color: theme.colors.text.secondary,
              margin: 0,
              textAlign: "center",
              opacity: transitionProgress,
              transform: `translateY(${interpolate(transitionProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            Root explained every install. Now it manages your environment.
          </p>
        </FadeIn>
      </div>
    </AbsoluteFill>
  );
};
