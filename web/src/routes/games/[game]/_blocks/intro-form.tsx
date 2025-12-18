import { handleHttpError } from "@api";
import { useGame, useGameIntroduction } from "@api/game";
import { type Article, ArticleAccessPolicy } from "@models/article";
import { createForm, required, setValues } from "@modular-forms/solid";
import { useParams } from "@solidjs/router";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Editor from "@widgets/editor";
import { DateTime } from "luxon";
import { createEffect, createSignal, untrack } from "solid-js";

type ArticleForm = {
  content: string;
};

export default function IntroForm(props: { onDone: (article: Article) => Promise<void> }) {
  const [loading, setLoading] = createSignal(false);
  const params = useParams();
  const gameId = Number.parseInt(params.game || "UNKN0WN", 10);
  const game = useGame({ id: () => gameId, enabled: () => !!gameId });
  const introduction = useGameIntroduction({ id: () => gameId, enabled: () => !!gameId });
  const [form, { Form, Field }] = createForm<ArticleForm>({
    initialValues: {
      content: introduction.data?.content || "",
    },
  });
  createEffect(() => {
    if (introduction.data) {
      untrack(() => {
        setValues(form, {
          content: introduction.data?.content || "",
        });
      });
    } else {
      untrack(() => {
        setValues(form, {
          content: undefined,
        });
      });
    }
  });

  async function onSubmit(result: ArticleForm) {
    setLoading(true);
    try {
      props.onDone({
        id: introduction.data?.id || 0,
        content: result.content,
        created_at: introduction.data?.created_at || DateTime.now(),
        updated_at: introduction.data?.updated_at || DateTime.now(),
        publisher_id: 0,
        access_policy: ArticleAccessPolicy.Game,
        draft: false,
        published: true,
        title: game.data?.name || "",
        path: [],
        enable_comment: false,
        weight: 0,
      });
    } catch (err) {
      handleHttpError(err as Error, t("general.actions.save.status.fail"));
    }
    setLoading(false);
  }
  return (
    <Form onSubmit={onSubmit} class="flex flex-col space-y-2 self-center w-full max-w-5xl flex-1">
      <Field name="content" validate={[required(t("game.form.introduction.required"))]}>
        {(field) => (
          <Editor
            form={form}
            lineNumbers
            class="flex-1"
            lang="markdown"
            placeholder="MARKDOWN"
            title={t("game.form.introduction.label")}
            name="content"
            value={field.value}
            error={field.error}
          />
        )}
      </Field>
      <Button type="submit" level="primary" class="mt-4!" loading={loading()} disabled={loading()}>
        {t("general.actions.save.title")}
      </Button>
    </Form>
  );
}
