import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { theme } from "../styles/theme";
import { Category } from "../data/catalog";

interface CategoryGridProps {
  categories: Category[];
  delay?: number;
  stagger?: number;
}

export const CategoryGrid: React.FC<CategoryGridProps> = ({
  categories,
  delay = 0,
  stagger = 5,
}) => {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(4, 1fr)",
        gap: 16,
        maxWidth: 800,
      }}
    >
      {categories.map((category, i) => (
        <CategoryCard
          key={i}
          category={category}
          delay={delay + i * stagger}
        />
      ))}
    </div>
  );
};

interface CategoryCardProps {
  category: Category;
  delay: number;
}

const CategoryCard: React.FC<CategoryCardProps> = ({ category, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 18, stiffness: 100, mass: 0.5 },
  });

  const opacity = progress;
  const scale = interpolate(progress, [0, 1], [0.9, 1]);
  const translateY = interpolate(progress, [0, 1], [10, 0]);

  return (
    <div
      style={{
        backgroundColor: theme.colors.bg.tertiary,
        borderRadius: theme.radius.md,
        border: `1px solid ${theme.colors.border.subtle}`,
        padding: 16,
        opacity,
        transform: `scale(${scale}) translateY(${translateY}px)`,
      }}
    >
      <div style={{ fontSize: 24, marginBottom: 8 }}>{category.icon}</div>
      <div
        style={{
          fontFamily: theme.fonts.sans,
          fontSize: 14,
          fontWeight: 600,
          color: theme.colors.text.primary,
          textTransform: "capitalize",
        }}
      >
        {category.name}
      </div>
      <div
        style={{
          fontFamily: theme.fonts.mono,
          fontSize: 12,
          color: theme.colors.text.tertiary,
          marginTop: 4,
        }}
      >
        {category.count} packages
      </div>
    </div>
  );
};
