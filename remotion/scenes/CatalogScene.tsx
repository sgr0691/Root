import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { FadeIn } from "../components/FadeIn";

export const CatalogScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const statsProgress = spring({
    frame: frame - 10,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const packages = [
    "terraform",
    "kubectl",
    "helm",
    "tmux",
    "neovim",
  ];

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
          maxWidth: 800,
        }}
      >
        <FadeIn delay={10}>
          <div
            style={{
              display: "flex",
              alignItems: "baseline",
              gap: 12,
              opacity: statsProgress,
              transform: `translateY(${interpolate(statsProgress, [0, 1], [10, 0])}px)`,
            }}
          >
            <span
              style={{
                fontFamily: theme.fonts.mono,
                fontSize: 64,
                fontWeight: 700,
                color: theme.colors.accent.blue,
              }}
            >
              42
            </span>
            <span
              style={{
                fontFamily: theme.fonts.sans,
                fontSize: 24,
                color: theme.colors.text.secondary,
              }}
            >
              curated developer tools
            </span>
          </div>
        </FadeIn>

        <div
          style={{
            display: "flex",
            gap: 12,
            flexWrap: "wrap",
            justifyContent: "center",
          }}
        >
          {packages.map((pkg, i) => (
            <FadeIn key={i} delay={25 + i * 4}>
              <div
                style={{
                  fontFamily: theme.fonts.mono,
                  fontSize: 15,
                  color: theme.colors.text.secondary,
                  padding: "6px 14px",
                  borderRadius: theme.radius.md,
                  backgroundColor: theme.colors.bg.tertiary,
                  border: `1px solid ${theme.colors.border.subtle}`,
                }}
              >
                {pkg}
              </div>
            </FadeIn>
          ))}
        </div>
      </div>
    </AbsoluteFill>
  );
};
