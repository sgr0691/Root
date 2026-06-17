import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";

interface BadgeProps {
  text: string;
  variant?: "success" | "info" | "warning" | "error" | "neutral";
  delay?: number;
  icon?: string;
}

export const Badge: React.FC<BadgeProps> = ({
  text,
  variant = "neutral",
  delay = 0,
  icon,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 15, stiffness: 120, mass: 0.5 },
  });

  const opacity = progress;
  const scale = interpolate(progress, [0, 1], [0.8, 1]);

  const variantStyles = {
    success: {
      bg: `${theme.colors.accent.green}15`,
      border: theme.colors.accent.green,
      text: theme.colors.accent.green,
    },
    info: {
      bg: `${theme.colors.accent.blue}15`,
      border: theme.colors.accent.blue,
      text: theme.colors.accent.blue,
    },
    warning: {
      bg: `${theme.colors.accent.amber}15`,
      border: theme.colors.accent.amber,
      text: theme.colors.accent.amber,
    },
    error: {
      bg: `${theme.colors.accent.red}15`,
      border: theme.colors.accent.red,
      text: theme.colors.accent.red,
    },
    neutral: {
      bg: `${theme.colors.text.secondary}15`,
      border: theme.colors.text.secondary,
      text: theme.colors.text.secondary,
    },
  };

  const style = variantStyles[variant];

  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 6,
        padding: "6px 14px",
        borderRadius: 99,
        backgroundColor: style.bg,
        border: `1px solid ${style.border}`,
        color: style.text,
        fontFamily: theme.fonts.sans,
        fontSize: 14,
        fontWeight: 500,
        letterSpacing: "0.01em",
        opacity,
        transform: `scale(${scale})`,
      }}
    >
      {icon && <span style={{ fontSize: 12 }}>{icon}</span>}
      {text}
    </span>
  );
};

interface BadgeGroupProps {
  badges: Array<{ text: string; variant?: BadgeProps["variant"]; icon?: string }>;
  delay?: number;
  stagger?: number;
}

export const BadgeGroup: React.FC<BadgeGroupProps> = ({
  badges,
  delay = 0,
  stagger = 10,
}) => {
  return (
    <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
      {badges.map((badge, i) => (
        <Badge
          key={i}
          text={badge.text}
          variant={badge.variant}
          icon={badge.icon}
          delay={delay + i * stagger}
        />
      ))}
    </div>
  );
};
