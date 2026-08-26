/** Minimal toast per UI_DESIGN_SPEC §29: dark pill, bottom center, auto-dismiss handled by caller. */
export function Toast({ message }: { message: string | null }) {
  if (!message) return null;
  return (
    <div
      role="status"
      className="fixed bottom-20 left-1/2 z-50 -translate-x-1/2 rounded-md bg-[#111111] px-4 py-3 text-sm text-white shadow-md animate-fade-in"
    >
      {message}
    </div>
  );
}
