import Terminal from "@lib/shell/terminal";
import type { Challenge } from "@models/challenge";

export default function (_props: { onStateChange?: (challenge?: Challenge) => void; inGame?: boolean }) {
  return <Terminal />;
}
