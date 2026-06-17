import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { FadeIn } from "../../components/FadeIn";

export const ThesisShort: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const thesisProgress = spring({
    frame: frame - 5,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const workflowProgress = spring({
    frame: frame - 45,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const titleProgress = spring({
    frame: frame - 70,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const urlProgress = spring({
    frame: frame - 90,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const workflow = ["Plan", "Install", "Verify", "Rollback"];

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
        }}
      >
        <div
          style={{
            textAlign: "center",
            opacity: thesisProgress,
            transform: `translateY(${interpolate(thesisProgress, [0, 1], [15, 0])}px)`,
          }}
        >
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 32,
              fontWeight: 500,
              color: theme.colors.text.tertiary,
              margin: 0,
              lineHeight: 1.4,
            }}
          >
            Package managers install.
          </p>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 36,
              fontWeight: 700,
              color: theme.colors.text.primary,
              margin: "8px 0 0 0",
              lineHeight: 1.4,
            }}
          >
            Root explains.
          </p>
        </div>

        <div
          style={{
            display: "flex",
            gap: 6,
            alignItems: "center",
            opacity: workflowProgress,
            transform: `translateY(${interpolate(workflowProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          {workflow.map((step, i) => (
            <React.Fragment key={i}>
              <div
                style={{
                  fontFamily: theme.fonts.sans,
                  fontSize: 16,
                  fontWeight: 600,
                  color: theme.colors.text.primary,
                  padding: "6px 14px",
                  borderRadius: theme.radius.md,
                  backgroundColor: theme.colors.bg.tertiary,
                  border: `1px solid ${theme.colors.border.subtle}`,
                }}
              >
                {step}
              </div>
              {i < workflow.length - 1 && (
                <span
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 14,
                    color: theme.colors.text.tertiary,
                  }}
                >
                  →
                </span>
              )}
            </React.Fragment>
          ))}
        </div>

        <h1
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 48,
            fontWeight: 800,
            color: theme.colors.text.primary,
            margin: 0,
            letterSpacing: "-0.03em",
            opacity: titleProgress,
            transform: `translateY(${interpolate(titleProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          Root v0.1.9
        </h1>

        <div
          style={{
            fontFamily: theme.fonts.mono,
            fontSize: 16,
            color: theme.colors.accent.blue,
            opacity: urlProgress,
            transform: `translateY(${interpolate(urlProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          github.com/sgr0691/Root
        </div>
      </div>
    </AbsoluteFill>
  );
};
