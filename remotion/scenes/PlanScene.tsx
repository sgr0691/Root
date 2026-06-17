import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { CommandRenderer, OutputLine } from "../components/CommandRenderer";

export const PlanScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const commandProgress = spring({
    frame,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const planBoxProgress = spring({
    frame: frame - 40,
    fps,
    config: { damping: 18, stiffness: 70, mass: 0.6 },
  });

  const glowProgress = spring({
    frame: frame - 55,
    fps,
    config: { damping: 30, stiffness: 50, mass: 1 },
  });

  const narrationProgress = spring({
    frame: frame - 210,
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
          gap: 48,
          width: "100%",
          maxWidth: 1100,
        }}
      >
        <div
          style={{
            opacity: commandProgress,
            transform: `translateY(${interpolate(commandProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          <CommandRenderer
            command="root plan install terraform"
            typingDelay={10}
            fontSize={36}
          />
        </div>

        <div
          style={{
            position: "relative",
            width: "100%",
          }}
        >
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
              padding: 48,
              width: "100%",
              opacity: planBoxProgress,
              transform: `translateY(${interpolate(planBoxProgress, [0, 1], [20, 0])}px)`,
              boxShadow: `0 0 100px rgba(59, 130, 246, ${0.12 * glowOpacity}), 0 0 40px rgba(59, 130, 246, ${0.06 * glowOpacity}), 0 25px 50px -12px rgba(0, 0, 0, 0.5)`,
            }}
          >
            <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 12,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 6,
                  }}
                >
                  Will install
                </div>
                <OutputLine
                  text="terraform"
                  delay={60}
                  color={theme.colors.text.primary}
                  fontSize={28}
                />
              </div>

              <div
                style={{
                  height: 1,
                  backgroundColor: theme.colors.border.subtle,
                  opacity: 0.5,
                }}
              />

              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 12,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 6,
                  }}
                >
                  Verification
                </div>
                <OutputLine
                  text="terraform version"
                  delay={75}
                  color={theme.colors.accent.blue}
                  fontSize={20}
                />
              </div>

              <div
                style={{
                  height: 1,
                  backgroundColor: theme.colors.border.subtle,
                  opacity: 0.5,
                }}
              />

              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 12,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 6,
                  }}
                >
                  History
                </div>
                <OutputLine
                  text="event recorded"
                  delay={90}
                  color={theme.colors.accent.green}
                  fontSize={20}
                  icon="✓"
                />
              </div>

              <div
                style={{
                  height: 1,
                  backgroundColor: theme.colors.border.subtle,
                  opacity: 0.5,
                }}
              />

              <div>
                <div
                  style={{
                    fontFamily: theme.fonts.mono,
                    fontSize: 12,
                    color: theme.colors.text.muted,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 6,
                  }}
                >
                  Rollback
                </div>
                <OutputLine
                  text="available"
                  delay={105}
                  color={theme.colors.accent.green}
                  fontSize={20}
                  icon="✓"
                />
              </div>
            </div>
          </div>
        </div>

        <p
          style={{
            fontFamily: theme.fonts.sans,
            fontSize: 24,
            color: theme.colors.text.secondary,
            margin: 0,
            textAlign: "center",
            opacity: narrationProgress,
            transform: `translateY(${interpolate(narrationProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          Root explains what is about to happen before it changes your machine.
        </p>
      </div>
    </AbsoluteFill>
  );
};
