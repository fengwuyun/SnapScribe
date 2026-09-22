import type { TranscriptSpeaker } from "@/types/transcript";

export function speakerDisplayName(
  speakerId: string | undefined,
  speakers: TranscriptSpeaker[],
): string {
  if (!speakerId) return "";
  return speakers.find((speaker) => speaker.id === speakerId)?.name.trim() ?? "";
}

const SPEAKER_BADGES = [
  "bg-[#EEF0FF] text-[#4E55C7]",
  "bg-[#F2ECFF] text-[#7552B8]",
  "bg-[#E8F7F4] text-[#25816F]",
  "bg-[#FFF2E8] text-[#A85A25]",
  "bg-[#EAF3FF] text-[#316FA8]",
  "bg-[#FFF0F5] text-[#A84E71]",
  "bg-[#F1F5E8] text-[#678233]",
  "bg-[#F3F3F5] text-[#626575]",
] as const;

export function speakerBadgeClass(colorIndex: number): string {
  return SPEAKER_BADGES[Math.abs(colorIndex) % SPEAKER_BADGES.length];
}

export type SpeakerDraftValidation =
  | { ok: false; message: string }
  | { ok: true; warning?: string };

export function validateSpeakerDrafts(speakers: TranscriptSpeaker[]): SpeakerDraftValidation {
  const names = speakers.map((speaker) => speaker.name.trim());
  if (names.some((name) => name.length === 0)) {
    return { ok: false, message: "说话人名称不能为空" };
  }
  if (new Set(names).size !== names.length) {
    return { ok: true, warning: "存在同名说话人，保存后仍会保持两个独立说话人" };
  }
  return { ok: true };
}
