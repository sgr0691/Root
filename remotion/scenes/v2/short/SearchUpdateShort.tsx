import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../../styles/theme";
import { CommandRenderer, OutputLine } from "../../../components/CommandRenderer";

export const SearchUpdateShort: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const dividerProgress = spring({
    frame: frame - 70,
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
          gap: 28,
          width: "100%",
          maxWidth: 900,
        }}
      >
        <CommandRenderer
          command="root search ripgrep"
          typingDelay={8}
          fontSize={26}
        />

        <div
          style={{
            backgroundColor: theme.colors.bg.tertiary,
            borderRadius: theme.radius.lg,
            border: `1px solid ${theme.colors.border.subtle}`,
            padding: 24,
            width: "100%",
          }}
        >
          <OutputLine
            text="ripgrep"
            delay={40}
            color={theme.colors.text.primary}
            fontSize={20}
            icon="📦"
          />
          <OutputLine
            text="v14.1.0 · terminal · rg"
            delay={50}
            color={theme.colors.text.secondary}
            fontSize={15}
          />
        </div>

        <div
          style={{
            width: "100%",
            height: 1,
            backgroundColor: theme.colors.border.subtle,
            opacity: dividerProgress,
          }}
        />

        <CommandRenderer
          command="root update"
          typingDelay={75}
          fontSize={26}
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
            text="Resolving..."
            delay={100}
            color={theme.colors.text.secondary}
            fontSize={16}
          />
          <OutputLine
            text="ripgrep 14.1.0 → 14.1.1"
            delay={115}
            color={theme.colors.accent.green}
            fontSize={16}
            icon="✓"
          />
          <OutputLine
            text="Verification complete"
            delay={130}
            color={theme.colors.accent.green}
            fontSize={16}
            icon="✓"
          />
        </div>
      </div>
    </AbsoluteFill>
  );
};
