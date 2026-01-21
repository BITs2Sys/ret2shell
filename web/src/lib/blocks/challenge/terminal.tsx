import Terminal from "@lib/shell/terminal";
import type { ChallengeWidgetProps } from ".";

export default function (props: ChallengeWidgetProps) {
  return <Terminal {...props} />;
}
