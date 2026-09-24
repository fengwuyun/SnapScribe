import { describe, expect, it, vi } from "vitest";
import { readDiarizationPreference, writeDiarizationPreference } from "@/lib/homePreferences";

describe("home preferences", () => {
  it("enables speaker diarization by default", () => {
    expect(readDiarizationPreference({ getItem: () => null })).toBe(true);
  });

  it("restores the last saved speaker diarization choice", () => {
    expect(readDiarizationPreference({ getItem: () => "false" })).toBe(false);
    expect(readDiarizationPreference({ getItem: () => "true" })).toBe(true);
  });

  it("persists changes without failing when storage is unavailable", () => {
    const setItem = vi.fn();
    writeDiarizationPreference(false, { setItem });
    expect(setItem).toHaveBeenCalledWith("snapscribe.diarization-enabled", "false");

    expect(() => writeDiarizationPreference(true, {
      setItem: () => { throw new Error("storage unavailable"); },
    })).not.toThrow();
  });
});
