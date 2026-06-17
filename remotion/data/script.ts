export interface SceneConfig {
  id: string;
  durationInFrames: number;
  title: string;
}

export const SCENES: SceneConfig[] = [
  { id: "Problem", durationInFrames: 135, title: "The Problem" },
  { id: "Plan", durationInFrames: 270, title: "Plan Before Install" },
  { id: "Install", durationInFrames: 180, title: "Install with Confidence" },
  { id: "Verify", durationInFrames: 150, title: "Verify" },
  { id: "Rollback", durationInFrames: 180, title: "Instant Rollback" },
  { id: "History", durationInFrames: 135, title: "Full History" },
  { id: "Catalog", durationInFrames: 90, title: "Curated Catalog" },
  { id: "Future", durationInFrames: 120, title: "What's Next" },
  { id: "Final", durationInFrames: 120, title: "Root v0.1.9" },
];

export const TOTAL_DURATION = SCENES.reduce(
  (sum, scene) => sum + scene.durationInFrames,
  0
);

export const FPS = 30;
export const WIDTH = 1920;
export const HEIGHT = 1080;

export type SceneId = (typeof SCENES)[number]["id"];
