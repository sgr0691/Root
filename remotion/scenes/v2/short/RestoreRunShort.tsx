import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../../styles/theme";
import { CommandRenderer, OutputLine } from "../../../components/CommandRenderer";
import { SplitScreen, SplitTerminal } from "../../../components/SplitScreen";

export const RestoreRunShort: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const dividerProgress = spring({
    frame: frame - 85,
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
          maxWidth: 1100,
        }}
      >
        <CommandRenderer
          command="root restore --lock root.lock"
          typingDelay={8}
          fontSize={24}
        />

        <SplitScreen leftDelay={40} rightDelay={55} gap={16}>
          <SplitTerminal title="root sync">
            <OutputLine
              text="Reconciling..."
              delay={60}
              color={theme.colors.text.secondary}
              fontSize={14}
            />
            <OutputLine
              text="Sync complete"
              delay={75}
              color={theme.colors.accent.green}
              fontSize={14}
              icon="✓"
            />
          </SplitTerminal>

          <SplitTerminal title="root restore">
            <OutputLine
              text="Restoring profile..."
              delay={75}
              color={theme.colors.text.secondary}
              fontSize={14}
            />
            <OutputLine
              text="Machine reproduced"
              delay={90}
              color={theme.colors.accent.green}
              fontSize={14}
              icon="✓"
            />
          </SplitTerminal>
        </SplitScreen>

        <div
          style={{
            width: "100%",
            height: 1,
            backgroundColor: theme.colors.border.subtle,
            opacity: dividerProgress,
          }}
        />

        <CommandRenderer
          command="root run dev"
          typingDelay={95}
          fontSize={24}
        />

        <div
          style={{
            backgroundColor: theme.colors.bg.tertiary,
            borderRadius: theme.radius.lg,
            border: `1px solid ${theme.colors.border.subtle}`,
            padding: 24,
            width: "100%",
            display: "flex",
            flexDirection: "column",
            gap: 8,
          }}
        >
          <OutputLine
            text="Executing task: dev"
            delay={120}
            color={theme.colors.text.secondary}
            fontSize={16}
          />
          <OutputLine
            text="Build successful"
            delay={140}
            color={theme.colors.accent.green}
            fontSize={16}
            icon="✓"
          />
          <OutputLine
            text="Task completed"
            delay={155}
            color={theme.colors.accent.green}
            fontSize={16}
            icon="✓"
          />
        </div>
      </div>
    </AbsoluteFill>
  );
};
