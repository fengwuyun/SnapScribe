export function appendWaveformLevel(
  levels: number[],
  next: number,
  limit = 64,
): number[] {
  return [...levels, Math.max(0, Math.min(1, next))].slice(-Math.max(1, limit));
}
