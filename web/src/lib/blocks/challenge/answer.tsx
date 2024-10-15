import { getChallengeAnswer, updateChallengeAnswer } from "@api/game";
import type { Challenge } from "@models/challenge";
import { challengeStore } from "@storage/challenge";
import { isGameAdmin } from "@storage/game";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Article from "@widgets/article";
import Button from "@widgets/button";
import { EditorBare } from "@widgets/editor";
import LoadingTips from "@widgets/loading-tips";
import type { HTTPError } from "ky";
import { createEffect, createSignal, Show, untrack } from "solid-js";

export default function(_props: {
  onStateChange?: (challenge?: Challenge) => void;
  inGame?: boolean;
}) {
  const [answer, setAnswer] = createSignal<string>("");
  const [loading, setLoading] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [inEdit, setInEdit] = createSignal(false);

  createEffect(() => {
    if (challengeStore.current) {
      untrack(() => {
        setLoading(true);
        getChallengeAnswer(challengeStore.current!.game_id, challengeStore.current!.id).then((data) => {
          setAnswer(data);
        }).catch((err: HTTPError) => {
          err.response.text().then(text => {
            addToast({
              level: "error",
              description: `${t("game.challenge.fetchFailed")}: ${text}`,
              duration: 5000
            })
          })
        }).finally(() => {
          setLoading(false);
        });
      })
    }
  })

  function handleUpdateAnswer() {
    setSubmitting(true);
    updateChallengeAnswer(challengeStore.current!.game_id, challengeStore.current!.id, answer()).then(() => {
      addToast({
        level: "success",
        description: t("form.saveSuccess")!,
        duration: 5000
      });
      setSubmitting(false);
      setInEdit(false);
      if (_props.onStateChange) _props.onStateChange();
    }).catch((err: HTTPError) => {
      err.response.text().then(text => {
        addToast({
          level: "error",
          description: `${t("form.saveFailed")}: ${text}`,
          duration: 5000
        })
      })
    })
  }

  return (
    <div class="min-h-full flex-1 flex flex-col space-y-2 p-3 lg:p-6 items-center">
      <header class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-2 font-bold w-full">
        <span class="icon-[fluent--book-20-regular] w-5 h-5 flex-shrink-0" />
        <span class="flex-1 text-start">{t("game.challenge.answer")}</span>
        <Show when={isGameAdmin()}>
          <Show when={!inEdit()} fallback={
            <Button size="sm" level='primary' onClick={handleUpdateAnswer} loading={submitting()} disabled={submitting()}>
              {t("form.save")}
            </Button>
          }>
            <Button size="sm" level='primary' onClick={() => {
              setInEdit(true);
            }}>
              {t("form.edit")}
            </Button>
          </Show>
        </Show>
      </header>
      <Show when={!inEdit()} fallback={
        <EditorBare
          class="flex-1 w-full"
          value={answer()}
          lang="markdown"
          lineNumbers
          onValueChanged={(v) => setAnswer(v)}
        />
      }>
        <Show when={!loading()} fallback={
          <article
            class={`article !max-w-5xl w-full`.trim()}
          >
            <p>
              <LoadingTips />
            </p>
          </article>
        }>
          <Article content={answer()} extra />
        </Show>
      </Show>
    </div>
  );
}
