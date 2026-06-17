import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";

export interface TimelineEvent {
  action: string;
  package: string;
  timestamp: string;
  type: "install" | "rollback" | "verify";
}

interface TimelineProps {
  events: TimelineEvent[];
  delay?: number;
  stagger?: number;
}

export const Timeline: React.FC<TimelineProps> = ({
  events,
  delay = 0,
  stagger = 15,
}) => {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 0,
        position: "relative",
      }}
    >
      {events.map((event, i) => (
        <TimelineItem
          key={i}
          event={event}
          delay={delay + i * stagger}
          isLast={i === events.length - 1}
        />
      ))}
    </div>
  );
};

interface TimelineItemProps {
  event: TimelineEvent;
  delay: number;
  isLast: boolean;
}

const TimelineItem: React.FC<TimelineItemProps> = ({ event, delay, isLast }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 20, stiffness: 100, mass: 0.5 },
  });

  const opacity = progress;
  const translateX = interpolate(progress, [0, 1], [20, 0]);

  const typeColors = {
    install: theme.colors.accent.green,
    rollback: theme.colors.accent.amber,
    verify: theme.colors.accent.blue,
  };

  const typeIcons = {
    install: "↓",
    rollback: "↩",
    verify: "✓",
  };

  const color = typeColors[event.type];
  const icon = typeIcons[event.type];

  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-start",
        opacity,
        transform: `translateX(${translateX}px)`,
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          marginRight: 16,
        }}
      >
        <div
          style={{
            width: 32,
            height: 32,
            borderRadius: "50%",
            backgroundColor: `${color}20`,
            border: `2px solid ${color}`,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 14,
            color,
            fontWeight: 600,
          }}
        >
          {icon}
        </div>
        {!isLast && (
          <div
            style={{
              width: 2,
              height: 40,
              backgroundColor: theme.colors.border.subtle,
            }}
          />
        )}
      </div>

      <div style={{ paddingTop: 4, paddingBottom: isLast ? 0 : 16 }}>
        <div
          style={{
            fontFamily: theme.fonts.mono,
            fontSize: 18,
            color: theme.colors.text.primary,
            fontWeight: 500,
          }}
        >
          {event.action}{" "}
          <span style={{ color: theme.colors.accent.blue }}>{event.package}</span>
        </div>
        <div
          style={{
            fontFamily: theme.fonts.mono,
            fontSize: 13,
            color: theme.colors.text.tertiary,
            marginTop: 2,
          }}
        >
          {event.timestamp}
        </div>
      </div>
    </div>
  );
};
