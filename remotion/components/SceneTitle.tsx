import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";

interface SceneTitleProps {
  title: string;
  subtitle?: string;
  delay?: number;
  align?: "left" | "center" | "right";
}

export const SceneTitle: React.FC<SceneTitleProps> = ({
  title,
  subtitle,
  delay = 0,
  align = "center",
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const titleProgress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const subtitleProgress = spring({
    frame: frame - delay - 10,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const titleOpacity = titleProgress;
  const titleY = interpolate(titleProgress, [0, 1], [20, 0]);

  const subtitleOpacity = subtitleProgress;
  const subtitleY = interpolate(subtitleProgress, [0, 1], [15, 0]);

  const textAlign = align === "center" ? "center" : align === "right" ? "right" : "left";

  return (
    <div style={{ textAlign, marginBottom: 40 }}>
      <h1
        style={{
          fontFamily: theme.fonts.sans,
          fontSize: 56,
          fontWeight: 700,
          color: theme.colors.text.primary,
          margin: 0,
          letterSpacing: "-0.03em",
          lineHeight: 1.1,
          opacity: titleOpacity,
          transform: `translateY(${titleY}px)`,
        }}
      >
        {title}
      </h1>
      {subtitle && (
        <p
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 22,
            color: theme.colors.text.secondary,
            marginTop: 12,
            fontWeight: 400,
            opacity: subtitleOpacity,
            transform: `translateY(${subtitleY}px)`,
          }}
        >
          {subtitle}
        </p>
      )}
    </div>
  );
};
