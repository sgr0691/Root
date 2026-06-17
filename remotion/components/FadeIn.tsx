import React from "react";
import { useCurrentFrame, interpolate, spring, useVideoConfig } from "remotion";

interface FadeInProps {
  children: React.ReactNode;
  delay?: number;
  duration?: number;
  direction?: "up" | "down" | "left" | "right" | "none";
  distance?: number;
}

export const FadeIn: React.FC<FadeInProps> = ({
  children,
  delay = 0,
  duration = 20,
  direction = "up",
  distance = 20,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    frame: frame - delay,
    fps,
    config: {
      damping: 20,
      stiffness: 100,
      mass: 0.5,
    },
  });

  const opacity = interpolate(progress, [0, 1], [0, 1]);

  let transform = "none";
  if (direction === "up") {
    transform = `translateY(${interpolate(progress, [0, 1], [distance, 0])}px)`;
  } else if (direction === "down") {
    transform = `translateY(${interpolate(progress, [0, 1], [-distance, 0])}px)`;
  } else if (direction === "left") {
    transform = `translateX(${interpolate(progress, [0, 1], [distance, 0])}px)`;
  } else if (direction === "right") {
    transform = `translateX(${interpolate(progress, [0, 1], [-distance, 0])}px)`;
  }

  return (
    <div style={{ opacity, transform }}>
      {children}
    </div>
  );
};
