import React from "react";
import { Sequence } from "remotion";
import { SCENES } from "./data/script";
import { ProblemScene } from "./scenes/ProblemScene";
import { PlanScene } from "./scenes/PlanScene";
import { InstallScene } from "./scenes/InstallScene";
import { VerifyScene } from "./scenes/VerifyScene";
import { RollbackScene } from "./scenes/RollbackScene";
import { HistoryScene } from "./scenes/HistoryScene";
import { CatalogScene } from "./scenes/CatalogScene";
import { FutureScene } from "./scenes/FutureScene";
import { FinalScene } from "./scenes/FinalScene";

export const RootLaunch: React.FC = () => {
  const getFrom = (sceneIndex: number) => {
    let from = 0;
    for (let i = 0; i < sceneIndex; i++) {
      from += SCENES[i].durationInFrames;
    }
    return from;
  };

  return (
    <>
      <Sequence from={getFrom(0)} durationInFrames={SCENES[0].durationInFrames}>
        <ProblemScene />
      </Sequence>

      <Sequence from={getFrom(1)} durationInFrames={SCENES[1].durationInFrames}>
        <PlanScene />
      </Sequence>

      <Sequence from={getFrom(2)} durationInFrames={SCENES[2].durationInFrames}>
        <InstallScene />
      </Sequence>

      <Sequence from={getFrom(3)} durationInFrames={SCENES[3].durationInFrames}>
        <VerifyScene />
      </Sequence>

      <Sequence from={getFrom(4)} durationInFrames={SCENES[4].durationInFrames}>
        <RollbackScene />
      </Sequence>

      <Sequence from={getFrom(5)} durationInFrames={SCENES[5].durationInFrames}>
        <HistoryScene />
      </Sequence>

      <Sequence from={getFrom(6)} durationInFrames={SCENES[6].durationInFrames}>
        <CatalogScene />
      </Sequence>

      <Sequence from={getFrom(7)} durationInFrames={SCENES[7].durationInFrames}>
        <FutureScene />
      </Sequence>

      <Sequence from={getFrom(8)} durationInFrames={SCENES[8].durationInFrames}>
        <FinalScene />
      </Sequence>
    </>
  );
};
