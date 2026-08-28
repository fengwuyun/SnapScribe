import { describe, expect, it } from "vitest";
import { appendWaveformLevel } from "@/lib/recordingUi";

describe("appendWaveformLevel", () => {
  it("keeps a stable-length waveform and clamps noisy input", () => {
    expect(appendWaveformLevel([0.2, 0.4], 2, 3)).toEqual([0.2, 0.4, 1]);
    expect(appendWaveformLevel([0.2, 0.4, 0.6], -1, 3)).toEqual([0.4, 0.6, 0]);
  });
});
