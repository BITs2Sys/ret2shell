import { useChallenge, useUpdateChallengeMutation } from "@api/challenge";
import type { Challenge } from "@models/challenge";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import { DateTime } from "luxon";
import { type ChallengeForm, FormBare } from "./form";

export default function (props: {
  onStateChange?: (challenge?: Challenge) => void;
  inGame?: boolean;
  gameId: number;
  challengeId: number;
}) {
  const challenge = useChallenge({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });

  const updateChallengeMutation = useUpdateChallengeMutation({
    onSuccess: () =>
      addToast({
        level: "success",
        description: t("general.actions.save.status.success"),
        duration: 5000,
      }),
  });

  async function handleUpdateChallenge(result: ChallengeForm) {
    const tags = result.tag.split("/").map((t) => {
      return { name: t, primary: false };
    });
    tags[0].primary = true;

    const data: Challenge = {
      ...challenge.data,
      id: challenge.data!.id,
      name: result.name,
      updated_at: DateTime.now(),
      content: result.content,
      game_id: props.gameId,
      tag: tags,
      hidden: challenge.data?.hidden ?? false,
      score: challenge.data?.score ?? result.initial,
      bucket: challenge.data!.bucket!,
      score_rule: {
        initial: result.initial ?? challengeStore.current?.score_rule.initial ?? 0,
        minimum: result.minimum ?? challengeStore.current?.score_rule.minimum ?? 0,
        decay: result.decay ?? challengeStore.current?.score_rule.decay ?? 0,
      },
      release_at: result.release_at ? DateTime.fromSeconds(result.release_at) : null,
      archive_at: result.archive_at ? DateTime.fromSeconds(result.archive_at) : null,
    };
    updateChallengeMutation.mutate({ game_id: props.gameId, challenge: data });
  }
  return (
    <div class="flex flex-col p-3 lg:p-6 w-full items-center">
      <header class="min-h-12 w-full max-w-5xl border-b border-b-layer-content/10 flex flex-row flex-wrap justify-end space-x-2 items-center gap-y-2 py-2 mb-2">
        <span class="flex flex-row space-x-2 items-center overflow-hidden">
          <span class="shrink-0 icon-[fluent--settings-20-regular] w-5 h-5" />
          <span class="font-bold inline-block whitespace-nowrap">{t("general.actions.edit.title")}</span>
        </span>
        <span class="flex-1" />
      </header>
      <FormBare
        onDone={handleUpdateChallenge}
        inGame={props.inGame}
        gameId={props.gameId}
        challengeId={props.challengeId}
      />
    </div>
  );
}
