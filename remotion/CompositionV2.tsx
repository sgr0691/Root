import React from "react";
import { Sequence } from "remotion";
import { V2_SCENES } from "./data/script-v2";
import { RecapScene } from "./scenes/v2/RecapScene";
import { SearchScene } from "./scenes/v2/SearchScene";
import { UpdateScene } from "./scenes/v2/UpdateScene";
import { RestoreScene } from "./scenes/v2/RestoreScene";
import { RunScene } from "./scenes/v2/RunScene";
import { PolicyScene } from "./scenes/v2/PolicyScene";
import { SandboxScene } from "./scenes/v2/SandboxScene";
import { StatusScene } from "./scenes/v2/StatusScene";
import { FinalSceneV2 } from "./scenes/v2/FinalSceneV2";

export const RootLaunchV2: React.FC = () => {
  const getFrom = (sceneIndex: number) => {
    let from = 0;
    for (let i = 0; i < sceneIndex; i++) {
      from += V2_SCENES[i].durationInFrames;
    }
    return from;
  };

  const scenes = [
    RecapScene,
    SearchScene,
    UpdateScene,
    RestoreScene,
    RunScene,
    PolicyScene,
    SandboxScene,
    StatusScene,
    FinalSceneV2,
  ];

  return (
    <>
      {scenes.map((SceneComponent, i) => (
        <Sequence
          key={V2_SCENES[i].id}
          from={getFrom(i)}
          durationInFrames={V2_SCENES[i].durationInFrames}
        >
          <SceneComponent />
        </Sequence>
      ))}
    </>
  );
};
