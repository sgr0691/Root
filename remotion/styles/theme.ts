export const theme = {
  colors: {
    bg: {
      primary: "#0a0a0a",
      secondary: "#111111",
      tertiary: "#1a1a1a",
      elevated: "#222222",
    },
    text: {
      primary: "#fafafa",
      secondary: "#a1a1a1",
      tertiary: "#737373",
      muted: "#525252",
    },
    accent: {
      blue: "#3b82f6",
      green: "#22c55e",
      amber: "#f59e0b",
      red: "#ef4444",
      purple: "#a855f7",
    },
    terminal: {
      bg: "#0d0d0d",
      border: "#262626",
      prompt: "#3b82f6",
      command: "#fafafa",
      output: "#a1a1a1",
      success: "#22c55e",
      error: "#ef4444",
    },
    border: {
      subtle: "#262626",
      medium: "#404040",
    },
  },
  fonts: {
    mono: "'JetBrains Mono', 'SF Mono', 'Fira Code', monospace",
    sans: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
  },
  spacing: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
    xxl: 48,
  },
  radius: {
    sm: 4,
    md: 8,
    lg: 12,
    xl: 16,
  },
} as const;

export type Theme = typeof theme;
