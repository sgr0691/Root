# Audio Tracks

Place audio files in this directory for optional use in the launch video.

## Expected Files

| File | Purpose | Timing |
|------|---------|--------|
| `narration.mp3` | Voiceover track | Full video (48.5s) |
| `narration-short.mp3` | Voiceover for X version | Short video (17s) |
| `music.mp3` | Background music | Full video, loops |
| `music-short.mp3` | Background music for X version | Short video, loops |
| `sfx/typing.mp3` | Terminal typing sound | Per keystroke |
| `sfx/success.mp3` | Success checkmark sound | On ✓ appearances |
| `sfx/transition.mp3` | Scene transition whoosh | Between scenes |

## Integration

Audio is **optional**. The video must work perfectly when muted.

To add audio to a scene:

```tsx
import { Audio } from "remotion";

<Audio src={staticFile("audio/music.mp3")} volume={0.3} />
```

For per-scene SFX:

```tsx
import { Audio, useCurrentFrame } from "remotion";

const frame = useCurrentFrame();
// Only play after frame 50
{frame >= 50 && <Audio src={staticFile("audio/sfx/success.mp3")} />}
```

## Guidelines

- Keep narration clear and calm — matches the "trustworthy, technical" brand
- Music should be ambient/lo-fi, not energetic or hype
- SFX should be subtle — viewers shouldn't notice them consciously
- All audio files should be short and compressed (mp3, 128kbps)
