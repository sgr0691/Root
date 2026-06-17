import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";
import { SplitScreen, SplitTerminal } from "../../components/SplitScreen";

export const RestoreScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const narrationProgress = spring({
    frame: frame - 120,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const narrationProgress2 = spring({
    frame: frame - 140,
    fps,
    config: { damping: 20, stiffness: 80, mass: 0.6 },
  });

  const convergeProgress = spring({
    frame: frame - 90,
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
          gap: 32,
          width: "100%",
          maxWidth: 1200,
        }}
      >
        <SplitScreen leftDelay={5} rightDelay={20}>
          <SplitTerminal title="Current Machine">
            <CommandRenderer
              command="root sync"
              typingDelay={5}
              typingSpeed={3}
              fontSize={16}
            />
            <OutputLine
              text="Reconciling lockfile..."
              delay={30}
              color={theme.colors.text.secondary}
              fontSize={15}
            />
            <OutputLine
              text="Checking profile state..."
              delay={45}
              color={theme.colors.text.secondary}
              fontSize={15}
            />
            <div
              style={{
                marginTop: 16,
                paddingTop: 16,
                borderTop: `1px solid ${theme.colors.border.subtle}`,
              }}
            >
              <OutputLine
                text="Sync complete"
                delay={60}
                color={theme.colors.accent.green}
                fontSize={16}
                icon="✓"
              />
            </div>
          </SplitTerminal>

          <SplitTerminal title="New Machine">
            <CommandRenderer
              command="root restore --lock root.lock"
              typingDelay={20}
              typingSpeed={3}
              fontSize={16}
            />
            <OutputLine
              text="Reading lockfile..."
              delay={55}
              color={theme.colors.text.secondary}
              fontSize={15}
            />
            <OutputLine
              text="Restoring profile..."
              delay={70}
              color={theme.colors.text.secondary}
              fontSize={15}
            />
            <div
              style={{
                marginTop: 16,
                paddingTop: 16,
                borderTop: `1px solid ${theme.colors.border.subtle}`,
              }}
            >
              <OutputLine
                text="Machine reproduced"
                delay={85}
                color={theme.colors.accent.green}
                fontSize={16}
                icon="✓"
              />
            </div>
          </SplitTerminal>
        </SplitScreen>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            opacity: convergeProgress,
            transform: `translateY(${interpolate(convergeProgress, [0, 1], [10, 0])}px)`,
          }}
        >
          <span
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 20,
              fontWeight: 600,
              color: theme.colors.accent.green,
            }}
          >
            Same result. Every time.
          </span>
        </div>

        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 4 }}>
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
            Reproduce any machine from a lockfile.
          </p>
          <p
            style={{
              fontFamily: theme.fonts.sans,
              fontSize: 18,
              color: theme.colors.text.tertiary,
              margin: 0,
              textAlign: "center",
              opacity: narrationProgress2,
              transform: `translateY(${interpolate(narrationProgress2, [0, 1], [10, 0])}px)`,
            }}
          >
            Sync or restore — same result every time.
          </p>
        </div>
      </div>
    </AbsoluteFill>
  );
};
