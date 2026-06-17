import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";

interface WorkflowPillsProps {
  steps: string[];
  delay?: number;
  stagger?: number;
  fontSize?: number;
}

export const WorkflowPills: React.FC<WorkflowPillsProps> = ({
  steps,
  delay = 0,
  stagger = 8,
  fontSize = 16,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const containerProgress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  return (
    <div
      style={{
        display: "flex",
        gap: 6,
        alignItems: "center",
        opacity: containerProgress,
        transform: `translateY(${interpolate(containerProgress, [0, 1], [10, 0])}px)`,
      }}
    >
      {steps.map((step, i) => {
        const pillProgress = spring({
          frame: frame - delay - i * stagger,
          fps,
          config: { damping: 20, stiffness: 100, mass: 0.5 },
        });

        return (
          <React.Fragment key={i}>
            <div
              style={{
                fontFamily: theme.fonts.sans,
                fontSize,
                fontWeight: 600,
                color: theme.colors.text.primary,
                padding: "6px 14px",
                borderRadius: theme.radius.md,
                backgroundColor: theme.colors.bg.tertiary,
                border: `1px solid ${theme.colors.border.subtle}`,
                opacity: pillProgress,
              }}
            >
              {step}
            </div>
            {i < steps.length - 1 && (
              <span
                style={{
                  fontFamily: theme.fonts.mono,
                  fontSize: fontSize - 2,
                  color: theme.colors.text.tertiary,
                  opacity: pillProgress,
                }}
              >
                →
              </span>
            )}
          </React.Fragment>
        );
      })}
    </div>
  );
};
