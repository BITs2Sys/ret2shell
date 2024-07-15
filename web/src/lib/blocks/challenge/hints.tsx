import { createChallengeHint, deleteChallengeHint, getChallengeHint, getTeamExtras } from "@api/game";
import type { Challenge } from "@models/challenge";
import { addToast } from "@storage/toast";
import type { Extra } from "@models/extra";
import type { Hint } from "@models/hint";
import { Permission } from "@models/user";
import { createForm, required, setValue, setValues } from "@modular-forms/solid";
import { accountStore } from "@storage/account";
import { gameStore, isGameAdmin } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Input from "@widgets/input";
import Popover from "@widgets/popover";
import type { HTTPError } from "ky";
import { LoremIpsum } from "lorem-ipsum";
import { DateTime } from "luxon";
import { For, Show, createEffect, createSignal, untrack } from "solid-js";

type CreateHintForm = {
  content: string;
  cost: number;
};

export default function (props: { challenge?: Challenge }) {
  const [hints, setHints] = createSignal([] as Hint[]);
  const [extras, setExtras] = createSignal([] as Extra[]);
  const lorem = new LoremIpsum({
    wordsPerSentence: {
      max: 8,
      min: 3,
    },
  });
  const [form, { Form, Field }] = createForm<CreateHintForm>();
  setValue(form, "cost", 0);
  const [loading, setLoading] = createSignal(false);
  function onSubmit(result: CreateHintForm) {
    const hint = {
      id: 0,
      created_at: DateTime.now(),
      challenge_id: props.challenge!.id,
      content: result.content,
      cost: result.cost || 0,
    } as Hint;
    createChallengeHint(props.challenge!.game_id, props.challenge!.id, hint)
      .then(() => {
        addToast({
          level: "success",
          description: t("form.createSuccess")!,
          duration: 5000,
        });
        refreshHint();
        setValues(form, {
          content: undefined,
          cost: undefined,
        });
      })
      .catch((e: HTTPError) => {
        e.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.createFailed")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
  function refreshHint() {
    setLoading(true);
    getChallengeHint(props.challenge!.game_id, props.challenge!.id)
      .then((resp) => {
        setHints(resp);
      })
      .catch((e: HTTPError) => {
        e.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.fetchHintFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setLoading(false);
      });
  }
  createEffect(() => {
    if (props.challenge) {
      untrack(() => {
        refreshHint();
        if (!isGameAdmin()) {
          getTeamExtras(gameStore.current!.id, gameStore.team!.id)
            .then((resp) => {
              setExtras(resp);
            })
            .catch((e: HTTPError) => {
              e.response.text().then((text) => {
                addToast({
                  level: "error",
                  description: `${t("game.challenge.fetchHintFailed")}: ${text}`,
                  duration: 5000,
                });
              });
            });
        }
      });
    }
  });

  function handleDeleteHint(id: number) {
    deleteChallengeHint(gameStore.current!.id, props.challenge!.id, id)
      .then(() => {
        refreshHint();
      })
      .catch((e: HTTPError) => {
        e.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.deleteFailed")}: ${text}`,
          });
        });
      });
  }
  return (
    <div class="flex flex-col p-3 lg:p-6">
      <For each={hints()}>
        {(hint) => (
          <div class="px-2 min-h-12 py-1 border-b border-b-layer-content/10 flex items-center space-x-2">
            <span class="icon-[fluent--info-20-regular] w-5 h-5 text-primary flex-shrink-0" />
            <Show
              when={isGameAdmin() || hint.cost === 0 || extras().find((e) => e.hint_id === hint.id)}
              fallback={
                <>
                  <span class="blur pointer-events-none select-none flex-1">{lorem.generateSentences(1)}</span>
                  <Popover
                    size="sm"
                    ghost
                    btnContent={
                      <>
                        <span class="text-warning">-{hint.cost}</span>
                        <span class="opacity-60 text-warning">pts</span>
                        <span class="icon-[fluent--lock-20-regular] w-5 h-5 text-warning" />
                      </>
                    }
                  >
                    <Card contentClass="p-2 flex flex-row items-center">
                      <span class="px-2">
                        {t("game.challenge.confirmUnlockHint", {
                          cost: hint.cost,
                        })}
                      </span>
                      <Button size="sm" level="error">
                        {t("platform.yes")}
                      </Button>
                    </Card>
                  </Popover>
                </>
              }
            >
              <span class="flex-1 text-start">{hint.content}</span>
              <Show when={hint.cost > 0}>
                <span class="text-success">-{hint.cost}</span>
                <span class="opacity-60 text-success">pts</span>
                <span class="icon-[fluent--lock-open-20-regular] w-5 h-5 text-success" />
              </Show>
            </Show>
            <Show when={isGameAdmin()}>
              <Button
                size="sm"
                ghost
                square
                onClick={() => {
                  handleDeleteHint(hint.id);
                }}
              >
                <span class="icon-[fluent--delete-20-regular] w-5 h-5" />
              </Button>
            </Show>
          </div>
        )}
      </For>
      <Show
        when={
          accountStore.permissions.includes(Permission.Game) && gameStore.current?.admins.includes(accountStore.id!)
        }
      >
        <Form onSubmit={onSubmit} class="px-2 min-h-12 border-b border-b-layer-content/10 flex items-center space-x-2">
          <span class="icon-[fluent--info-20-regular] w-5 h-5 text-primary flex-shrink-0" />
          <Field name="content" validate={[required(t("game.challenge.hintRequired")!)]}>
            {(field, props) => (
              <Input
                type="text"
                value={field.value}
                error={field.error}
                {...props}
                required
                noLabel
                placeholder={t("game.challenge.createHint")}
                class="flex-1"
                size="sm"
              />
            )}
          </Field>
          <Field name="cost" type="number">
            {(field, props) => (
              <Input
                type="number"
                value={field.value}
                error={field.error}
                {...props}
                noLabel
                placeholder={t("game.challenge.createHintCost")}
                class="w-24"
                size="sm"
              />
            )}
          </Field>
          <span class="font-bold opacity-60">pts</span>
          <span class="w-8" />
          <Button size="sm" level="primary" type="submit" loading={loading()} disabled={loading()}>
            <span class="icon-[fluent--add-20-regular] w-5 h-5" />
            <span>{t("form.create")}</span>
          </Button>
        </Form>
      </Show>
    </div>
  );
}
