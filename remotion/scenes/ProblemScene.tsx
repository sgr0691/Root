import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { TerminalPlayback } from "../components/TerminalPlayback";
import { FadeIn } from "../components/FadeIn";

export const ProblemScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const nowWhatProgress = spring({
    frame: frame - 35,
    fps,
    config: { damping: 15, stiffness: 100, mass: 0.4 },
  });

  const questions = [
    "Is it working?",
    "Where did it install?",
    "Can I undo it?",
    "What changed?",
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
          gap: 32,
          width: "100%",
          maxWidth: 1200,
        }}
      >
        <TerminalPlayback
          lines={[
            { text: "brew install terraform", type: "command", delay: 0 },
            { text: "==> Downloading terraform...", type: "output", delay: 15 },
            { text: "==> Installing terraform...", type: "output", delay: 25 },
            { text: "Installed.", type: "success", delay: 35 },
          ]}
          width={750}
          height={280}
        />

        <div
          style={{
            opacity: nowWhatProgress,
            transform: `translateY(${interpolate(nowWhatProgress, [0, 1], [10, 0])}px)`,
            fontFamily: theme.fonts.sans,
            fontSize: 28,
            fontWeight: 600,
            color: theme.colors.accent.amber,
          }}
        >
          Now what?
        </div>

        <div
          style={{
            display: "flex",
            gap: 16,
            flexWrap: "wrap",
            justifyContent: "center",
          }}
        >
          {questions.map((q, i) => (
            <FadeIn key={i} delay={45 + i * 8}>
              <div
                style={{
                  fontFamily: theme.fonts.mono,
                  fontSize: 15,
                  color: theme.colors.accent.amber,
                  padding: "6px 14px",
                  borderRadius: theme.radius.md,
                  backgroundColor: `${theme.colors.accent.amber}10`,
                  border: `1px solid ${theme.colors.accent.amber}30`,
                }}
              >
                {q}
              </div>
            </FadeIn>
          ))}
        </div>
      </div>
    </AbsoluteFill>
  );
};
