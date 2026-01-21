import { useGame, useUpdateGameMutation } from "@api/game";
import { createForm, required } from "@modular-forms/solid";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Input from "@widgets/input";
import Popover from "@widgets/popover";
import TimePicker from "@widgets/timepicker";
import { DateTime } from "luxon";
import { createMemo, For } from "solid-js";

type TimelinePresetFormType = {
  label: string;
  start_at: number;
  end_at: number;
};

type TimelinePreset = {
  label: string;
  start_at: DateTime;
  end_at: DateTime;
};

export default function Timeline() {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  const game = useGame({ id: gameId, enabled: () => gameId() > 0 });
  const presets = createMemo(() => (game.data?.timeline_presets ?? []) as TimelinePreset[]);

  const updateMutation = useUpdateGameMutation({
    onSuccess: () => {
      game.refetch();
    },
  });
  const [form, { Form, Field }] = createForm<TimelinePresetFormType>();

  async function handleDeleteTimelinePreset(p: TimelinePreset) {
    if (!game.data) return;
    updateMutation.mutate({
      id: game.data.id,
      game: {
        ...game.data,
        timeline_presets: presets().filter((preset) => !(p.start_at === preset.start_at && p.end_at === preset.end_at)),
      },
    });
  }

  async function onSubmit(result: TimelinePresetFormType) {
    const results = [
      ...presets(),
      {
        label: result.label,
        start_at: DateTime.fromSeconds(result.start_at),
        end_at: DateTime.fromSeconds(result.end_at),
      },
    ].sort((a, b) => a.start_at.toSeconds() - b.start_at.toSeconds());
    if (!game.data) return;
    updateMutation.mutate({
      id: game.data.id,
      game: {
        ...game.data,
        timeline_presets: results,
      },
    });
  }
  return (
    <>
      <Title page={t("game.timeline.title")} route={`/games/${gameId()}/admin/timeline`} />
      <div class="flex flex-col space-y-4 xl:flex-row xl:space-y-0 xl:space-x-4 p-3 lg:p-6 w-full">
        <Form onSubmit={onSubmit} class="flex flex-col space-y-2">
          <Field name="label" validate={[required(t("game.timeline.form.label.required"))]}>
            {(field, props) => (
              <Input
                icon={<span class="shrink-0 icon-[fluent--number-symbol-20-regular] w-5 h-5" />}
                placeholder={t("game.timeline.form.label.placeholder")}
                title={t("game.timeline.form.label.label")}
                {...props}
                value={field.value}
                error={field.error}
                required
              />
            )}
          </Field>
          <Field name="start_at" type="number" validate={[required(t("game.timeline.form.startAt.required"))]}>
            {(startAtField) => (
              <Field name="end_at" type="number" validate={[required(t("game.timeline.form.endAt.required"))]}>
                {(endAtField) => (
                  <TimePicker
                    form={form}
                    type="time"
                    range
                    title={t("game.timeline.form.startEndTime.label")}
                    placeholder={t("game.timeline.form.startEndTime.placeholder")}
                    name={startAtField.name}
                    value={startAtField.value}
                    nameNext={endAtField.name}
                    valueNext={endAtField.value}
                    error={startAtField.error || endAtField.error}
                    startEdge={game.data?.start_at}
                    endEdge={game.data?.end_at}
                  />
                )}
              </Field>
            )}
          </Field>
          <Button
            type="submit"
            level="primary"
            class="mt-4!"
            loading={updateMutation.isPending}
            disabled={updateMutation.isPending}
          >
            {t("general.actions.create.title")}
          </Button>
        </Form>
        <div class="flex flex-col flex-1 py-8">
          <For
            each={presets()}
            fallback={
              <div class="h-full flex flex-col opacity-60 items-center justify-center space-y-8">
                <span class="shrink-0 icon-[fluent--archive-20-regular] w-24 h-24" />
                <span>{t("game.timeline.empty")}</span>
              </div>
            }
          >
            {(preset) => (
              <div class="flex flex-row w-full items-center">
                <div class="border-l-2 border-l-layer-content/20 border-dashed w-6 h-full flex items-center justify-center">
                  <div class="w-1 h-1 rounded-full bg-primary" />
                </div>
                <Card class="flex-1 m-2" contentClass="p-2 flex flex-row items-center space-x-2">
                  <span class="shrink-0 icon-[fluent--clock-20-regular] w-5 h-5" />
                  <span class="flex-1 text-start truncate">{preset.label}</span>
                  <span class="opacity-60">
                    {preset.start_at.toFormat("yyyy-MM-dd HH:mm")} - {preset.end_at.toFormat("yyyy-MM-dd HH:mm")}
                  </span>
                  <Popover
                    ghost
                    size="sm"
                    square
                    btnContent={<span class="shrink-0 icon-[fluent--delete-20-regular] w-5 h-5 text-error" />}
                  >
                    <Card contentClass="p-2 flex flex-col space-x-2 max-w-96">
                      <span class="inline-block space-x-2">
                        <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5 text-warning align-middle" />
                        <span>{t("game.timeline.deleteWarning")}</span>
                      </span>
                      <Button
                        level="primary"
                        size="sm"
                        class="self-end"
                        onClick={() => handleDeleteTimelinePreset(preset)}
                        loading={updateMutation.isPending}
                      >
                        {t("general.actions.yes.title")}
                      </Button>
                    </Card>
                  </Popover>
                </Card>
              </div>
            )}
          </For>
        </div>
      </div>
    </>
  );
}
