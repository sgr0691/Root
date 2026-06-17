import React from "react";
import { Sequence } from "remotion";
import { V2_SHORT_SCENES } from "./data/script-v2";
import { RecapShort } from "./scenes/v2/short/RecapShort";
import { SearchUpdateShort } from "./scenes/v2/short/SearchUpdateShort";
import { RestoreRunShort } from "./scenes/v2/short/RestoreRunShort";
import { FinalShortV2 } from "./scenes/v2/short/FinalShortV2";

export const RootLaunchV2Short: React.FC = () => {
  const getFrom = (sceneIndex: number) => {
    let from = 0;
    for (let i = 0; i < sceneIndex; i++) {
      from += V2_SHORT_SCENES[i].durationInFrames;
    }
    return from;
  };

  const scenes = [
    RecapShort,
    SearchUpdateShort,
    RestoreRunShort,
    FinalShortV2,
  ];

  return (
    <>
      {scenes.map((SceneComponent, i) => (
        <Sequence
          key={V2_SHORT_SCENES[i].id}
          from={getFrom(i)}
          durationInFrames={V2_SHORT_SCENES[i].durationInFrames}
        >
          <SceneComponent />
        </Sequence>
      ))}
    </>
  );
};
