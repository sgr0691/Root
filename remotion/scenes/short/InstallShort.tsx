import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../../styles/theme";
import { CommandRenderer, OutputLine } from "../../components/CommandRenderer";

export const InstallShort: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

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
          maxWidth: 800,
        }}
      >
        <CommandRenderer
          command="root install terraform"
          typingDelay={8}
          fontSize={32}
        />

        <div
          style={{
            backgroundColor: theme.colors.bg.tertiary,
            borderRadius: theme.radius.lg,
            border: `1px solid ${theme.colors.border.subtle}`,
            padding: 28,
            width: "100%",
            display: "flex",
            flexDirection: "column",
            gap: 10,
          }}
        >
          <OutputLine
            text="Installed"
            delay={40}
            color={theme.colors.accent.green}
            fontSize={22}
            icon="✓"
          />
          <OutputLine
            text="Verified"
            delay={52}
            color={theme.colors.accent.green}
            fontSize={22}
            icon="✓"
          />
          <OutputLine
            text="Rollback Available"
            delay={64}
            color={theme.colors.accent.green}
            fontSize={22}
            icon="✓"
          />
        </div>
      </div>
    </AbsoluteFill>
  );
};
