export function hasTextSelection(selection: Pick<Selection, "toString"> | null | undefined) {
  return Boolean(selection?.toString().trim());
}
