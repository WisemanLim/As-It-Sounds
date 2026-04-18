export default function toast(message: string) {
  if (typeof window === "undefined") return;
  console.warn("[toast]", message);
  try {
    const el = document.createElement("div");
    el.textContent = message;
    el.className =
      "fixed bottom-6 left-1/2 -translate-x-1/2 rounded-xl bg-red-900/80 px-4 py-2 text-sm text-red-100 shadow-lg backdrop-blur z-50";
    document.body.appendChild(el);
    setTimeout(() => el.remove(), 4000);
  } catch {
    // ignore DOM errors
  }
}
