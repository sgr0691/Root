import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { Logo } from "../../components/Logo";

export const FinalSceneV2: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoProgress = spring({
    frame: frame - 5,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const thesisProgress = spring({
    frame: frame - 15,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const urlProgress = spring({
    frame: frame - 50,
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
          maxWidth: 1000,
        }}
      >
        <div
          style={{
            opacity: logoProgress,
            transform: `translateY(${interpolate(logoProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          <Logo size="large" />
        </div>

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
              fontSize: 28,
              fontWeight: 500,
              color: theme.colors.text.tertiary,
              margin: 0,
              lineHeight: 1.5,
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
              margin: "6px 0 0 0",
              lineHeight: 1.4,
            }}
          >
            Root explains.
          </p>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 36,
              fontWeight: 700,
              color: theme.colors.text.primary,
              margin: "4px 0 0 0",
              lineHeight: 1.4,
            }}
          >
            Root restores.
          </p>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 40,
              fontWeight: 800,
              color: theme.colors.accent.blue,
              margin: "4px 0 0 0",
              lineHeight: 1.4,
            }}
          >
            Root manages.
          </p>
        </div>

        <div
          style={{
            position: "absolute",
            bottom: 80,
            left: "50%",
            transform: "translateX(-50%)",
            fontFamily: theme.fonts.mono,
            fontSize: 18,
            color: theme.colors.accent.blue,
            opacity: urlProgress,
          }}
        >
          github.com/sgr0691/Root
        </div>
      </div>
    </AbsoluteFill>
  );
};
