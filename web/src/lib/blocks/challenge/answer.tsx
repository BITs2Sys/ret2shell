import { handleHttpError } from "@api";
import { getChallengeAnswer, updateChallengeAnswer } from "@api/game";
import type { Challenge } from "@models/challenge";
import { challengeStore } from "@storage/challenge";
import { isGameAdmin } from "@storage/game";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import { useMutation, useQuery } from "@tanstack/solid-query";
import Article from "@widgets/article";
import Button from "@widgets/button";
import { EditorBare } from "@widgets/editor";
import LoadingTips from "@widgets/loading-tips";
import { createSignal, Show, Suspense } from "solid-js";

export default function (props: { onStateChange?: (challenge?: Challenge) => void; inGame?: boolean }) {
  const [answer, setAnswer] = createSignal("");
  const [inEdit, setInEdit] = createSignal(false);

  const answerQuery = useQuery(() => ({
    queryKey: ["game", challengeStore.current?.game_id, "challenge", challengeStore.current?.id, "answer"],
    queryFn: async () => await getChallengeAnswer(challengeStore.current!.game_id, challengeStore.current!.id),
    enabled: !!challengeStore.current,
    throwOnError: (err: Error) => {
      handleHttpError(err, t("challenge.answer.errors.fetchAnswer.title")!);
      return false;
    },
  }));

  const updateAnswerMutation = useMutation(() => ({
    mutationFn: (newAnswer: string) => {
      if (!challengeStore.current) {
        return Promise.reject("No challenge selected");
      }
      return updateChallengeAnswer(challengeStore.current!.game_id, challengeStore.current!.id, newAnswer);
    },
    onSuccess: () => {
      addToast({
        level: "success",
        description: t("general.actions.save.status.success")!,
        duration: 5000,
      });
      setInEdit(false);
      if (props.onStateChange) props.onStateChange();
      answerQuery.refetch();
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.save.status.fail")!);
    },
  }));

  return (
    <div class="min-h-full flex-1 flex flex-col space-y-2 p-3 lg:p-6 items-center">
      <header class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-2 font-bold w-full">
        <span class="shrink-0 icon-[fluent--book-20-regular] w-5 h-5" />
        <span class="flex-1 text-start">{t("challenge.answer.title")}</span>
        <Show when={isGameAdmin()}>
          <Show
            when={!inEdit()}
            fallback={
              <Button
                size="sm"
                level="primary"
                onClick={() => updateAnswerMutation.mutate(answer())}
                loading={updateAnswerMutation.isPending}
                disabled={updateAnswerMutation.isPending}
              >
                {t("general.actions.save.title")}
              </Button>
            }
          >
            <Button
              size="sm"
              level="primary"
              onClick={() => {
                setInEdit(true);
              }}
            >
              {t("general.actions.edit.title")}
            </Button>
          </Show>
        </Show>
      </header>
      <Show
        when={!inEdit()}
        fallback={
          <EditorBare
            class="flex-1 w-full"
            value={answerQuery.data}
            lang="markdown"
            lineNumbers
            onValueChanged={(v) => setAnswer(v)}
          />
        }
      >
        <Suspense
          fallback={
            <article class="article !max-w-5xl w-full">
              <p>
                <LoadingTips />
              </p>
            </article>
          }
        >
          <Article content={answerQuery.data || ""} extra />
        </Suspense>
      </Show>
    </div>
  );
}
