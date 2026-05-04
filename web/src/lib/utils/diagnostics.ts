import type { DiagnosticMarker } from "@widgets/editor";

export function hasDiagnosticErrors(lint?: DiagnosticMarker[] | null) {
  return lint?.some((marker) => marker.kind === "error") ?? false;
}
