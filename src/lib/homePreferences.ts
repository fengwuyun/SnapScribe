const DIARIZATION_STORAGE_KEY = "snapscribe.diarization-enabled";

type PreferenceReader = Pick<Storage, "getItem">;
type PreferenceWriter = Pick<Storage, "setItem">;

function browserStorage(): Storage | undefined {
  return typeof window === "undefined" ? undefined : window.localStorage;
}

export function readDiarizationPreference(storage: PreferenceReader | undefined = browserStorage()): boolean {
  try {
    return storage?.getItem(DIARIZATION_STORAGE_KEY) !== "false";
  } catch {
    return true;
  }
}

export function writeDiarizationPreference(
  enabled: boolean,
  storage: PreferenceWriter | undefined = browserStorage(),
): void {
  try {
    storage?.setItem(DIARIZATION_STORAGE_KEY, String(enabled));
  } catch {
    // Preferences remain usable for the current session when storage is unavailable.
  }
}
