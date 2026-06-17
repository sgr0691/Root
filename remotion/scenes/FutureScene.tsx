import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { FadeIn } from "../components/FadeIn";

export const FutureScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const features = [
    { label: "Permissions", icon: "🔐" },
    { label: "Sandboxes", icon: "📦" },
    { label: "Agent Runtime", icon: "🤖" },
  ];

  const titleProgress = spring({
    frame: frame - 10,
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
          maxWidth: 800,
        }}
      >
        <FadeIn delay={10}>
          <div
            style={{
              opacity: titleProgress,
              transform: `translateY(${interpolate(titleProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            <span
              style={{
                fontFamily: theme.fonts.mono,
                fontSize: 14,
                color: theme.colors.text.tertiary,
                textTransform: "uppercase",
                letterSpacing: "0.1em",
              }}
            >
              Tomorrow
            </span>
          </div>
        </FadeIn>

        <div
          style={{
            display: "flex",
            gap: 20,
            justifyContent: "center",
          }}
        >
          {features.map((feature, i) => (
            <FadeIn key={i} delay={25 + i * 15} direction="up" distance={15}>
              <div
                style={{
                  backgroundColor: theme.colors.bg.tertiary,
                  borderRadius: theme.radius.lg,
                  border: `1px solid ${theme.colors.border.subtle}`,
                  padding: 24,
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  gap: 12,
                  minWidth: 160,
                }}
              >
                <span style={{ fontSize: 28 }}>{feature.icon}</span>
                <span
                  style={{
                    fontFamily: theme.fonts.sans,
                    fontSize: 18,
                    fontWeight: 600,
                    color: theme.colors.text.primary,
                  }}
                >
                  {feature.label}
                </span>
              </div>
            </FadeIn>
          ))}
        </div>
      </div>
    </AbsoluteFill>
  );
};
