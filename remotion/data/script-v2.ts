export interface SceneConfig {
  id: string;
  durationInFrames: number;
  title: string;
}

export const V2_SCENES: SceneConfig[] = [
  { id: "Recap",   durationInFrames: 90,  title: "Recap" },
  { id: "Search",  durationInFrames: 120, title: "Search" },
  { id: "Update",  durationInFrames: 150, title: "Update" },
  { id: "Restore", durationInFrames: 210, title: "Restore" },
  { id: "Run",     durationInFrames: 180, title: "Run" },
  { id: "Policy",  durationInFrames: 180, title: "Policies" },
  { id: "Sandbox", durationInFrames: 210, title: "Sandboxes" },
  { id: "Status",  durationInFrames: 120, title: "Status" },
  { id: "Final",   durationInFrames: 180, title: "Finale" },
];

export const V2_TOTAL_DURATION = V2_SCENES.reduce(
  (sum, scene) => sum + scene.durationInFrames, 0
);

export const V2_FPS = 30;
export const V2_WIDTH = 1920;
export const V2_HEIGHT = 1080;

export const V2_SHORT_SCENES: SceneConfig[] = [
  { id: "RecapShort",     durationInFrames: 60,  title: "Recap" },
  { id: "SearchUpdateShort", durationInFrames: 150, title: "Search + Update" },
  { id: "RestoreRunShort",   durationInFrames: 180, title: "Restore + Run" },
  { id: "FinalShort",     durationInFrames: 150, title: "Finale" },
];

export const V2_SHORT_TOTAL_DURATION = V2_SHORT_SCENES.reduce(
  (sum, s) => sum + s.durationInFrames, 0
);
