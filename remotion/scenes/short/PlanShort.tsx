import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const PlanShort: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const boxProgress = spring({
    frame: frame - 30,
    fps,
    config: { damping: 18, stiffness: 70, mass: 0.6 },
  });

  const glowProgress = spring({
    frame: frame - 42,
    fps,
    config: { damping: 30, stiffness: 50, mass: 1 },
  });

  const narrationProgress = spring({
    frame: frame - 110,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const glowOpacity = interpolate(glowProgress, [0, 1], [0, 1]);

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
          maxWidth: 1000,
        }}
      >
        <CommandRenderer
          command="root plan install terraform"
          typingDelay={8}
          fontSize={32}
        />

        <div style={{ position: "relative", width: "100%" }}>
          <div
            style={{
              position: "absolute",
              inset: -4,
              borderRadius: theme.radius.xl,
              background: `radial-gradient(ellipse at center, rgba(59, 130, 246, ${0.18 * glowOpacity}) 0%, transparent 70%)`,
              filter: `blur(${50 * glowOpacity}px)`,
              pointerEvents: "none",
            }}
          />

          <div
            style={{
              position: "relative",
              backgroundColor: theme.colors.bg.secondary,
              borderRadius: theme.radius.xl,
              border: `1.5px solid rgba(59, 130, 246, ${0.35 * glowOpacity})`,
              padding: 36,
              opacity: boxProgress,
              transform: `translateY(${interpolate(boxProgress, [0, 1], [15, 0])}px)`,
              boxShadow: `0 0 100px rgba(59, 130, 246, ${0.12 * glowOpacity}), 0 0 40px rgba(59, 130, 246, ${0.06 * glowOpacity})`,
            }}
          >
            <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 11,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 4,
                  }}
                >
                  Will install
                </div>
                <OutputLine
                  text="terraform"
                  delay={50}
                  color={theme.colors.text.primary}
                  fontSize={24}
                />
              </div>

              <div style={{ height: 1, backgroundColor: theme.colors.border.subtle, opacity: 0.5 }} />

              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 11,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 4,
                  }}
                >
                  Verification
                </div>
                <OutputLine
                  text="terraform version"
                  delay={60}
                  color={theme.colors.accent.blue}
                  fontSize={18}
                />
              </div>

              <div style={{ height: 1, backgroundColor: theme.colors.border.subtle, opacity: 0.5 }} />

              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 11,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 4,
                  }}
                >
                  History
                </div>
                <OutputLine
                  text="event recorded"
                  delay={70}
                  color={theme.colors.accent.green}
                  fontSize={18}
                  icon="✓"
                />
              </div>

              <div style={{ height: 1, backgroundColor: theme.colors.border.subtle, opacity: 0.5 }} />

              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 11,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 4,
                  }}
                >
                  Rollback
                </div>
                <OutputLine
                  text="available"
                  delay={80}
                  color={theme.colors.accent.green}
                  fontSize={18}
                  icon="✓"
                />
              </div>
            </div>
          </div>
        </div>

        <p
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 20,
            color: theme.colors.text.secondary,
            margin: 0,
            textAlign: "center",
            opacity: narrationProgress,
            transform: `translateY(${interpolate(narrationProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          Root explains before it changes your machine.
        </p>
      </div>
    </AbsoluteFill>
  );
};
