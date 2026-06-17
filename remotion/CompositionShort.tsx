import React from "react";
import { Sequence } from "remotion";
import { ProblemShort } from "./scenes/short/ProblemShort";
import { PlanShort } from "./scenes/short/PlanShort";
import { InstallShort } from "./scenes/short/InstallShort";
import { ThesisShort } from "./scenes/short/ThesisShort";

export const SHORT_SCENES = [
  { id: "Problem", durationInFrames: 120 },
  { id: "Plan", durationInFrames: 165 },
  { id: "Install", durationInFrames: 120 },
  { id: "Thesis", durationInFrames: 120 },
];

export const SHORT_TOTAL_DURATION = SHORT_SCENES.reduce(
  (sum, s) => sum + s.durationInFrames,
  0
);

export const RootLaunchShort: React.FC = () => {
  const getFrom = (index: number) => {
    let from = 0;
    for (let i = 0; i < index; i++) {
      from += SHORT_SCENES[i].durationInFrames;
    }
    return from;
  };

  return (
    <>
      <Sequence from={getFrom(0)} durationInFrames={SHORT_SCENES[0].durationInFrames}>
        <ProblemShort />
      </Sequence>

      <Sequence from={getFrom(1)} durationInFrames={SHORT_SCENES[1].durationInFrames}>
        <PlanShort />
      </Sequence>

      <Sequence from={getFrom(2)} durationInFrames={SHORT_SCENES[2].durationInFrames}>
        <InstallShort />
      </Sequence>

      <Sequence from={getFrom(3)} durationInFrames={SHORT_SCENES[3].durationInFrames}>
        <ThesisShort />
      </Sequence>
    </>
  );
};
