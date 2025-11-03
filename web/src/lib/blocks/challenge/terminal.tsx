import Terminal from "@lib/shell/terminal";
import type { Challenge } from "@models/challenge";

export default function (props: {
  onStateChange?: (challenge?: Challenge) => void;
  inGame?: boolean;
  gameId: number;
  challengeId: number;
}) {
  return <Terminal {...props} />;
}
