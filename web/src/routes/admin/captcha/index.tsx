import { usePlatformConfig, useUpdatePlatformConfigMutation } from "@api/platform";
import type { CaptchaConfig, Config } from "@models/config";
import { createForm, getValue, required, setValues } from "@modular-forms/solid";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Checkbox from "@widgets/checkbox";
import Select from "@widgets/select";
import Slider from "@widgets/slider";
import { createEffect } from "solid-js";

export default function () {
  const [form, { Form, Field }] = createForm<CaptchaConfig>();
  const config = usePlatformConfig();
  const mutation = useUpdatePlatformConfigMutation({
    onSuccess: () => {
      addToast({
        level: "success",
        description: t("general.actions.save.status.success"),
        duration: 5000,
      });
    },
  });

  createEffect(() => {
    if (config.data)
      setValues(form, {
        enabled: config.data.captcha.enabled,
        difficulty: config.data.captcha.difficulty,
        validator: config.data.captcha.validator,
      });
  });

  async function onSubmit(result: CaptchaConfig) {
    const mergedConfig = {
      ...config.data,
      captcha: {
        enabled: result.enabled,
        difficulty: result.difficulty,
        validator: result.validator,
      },
    } as Config;
    mutation.mutate(mergedConfig);
  }
  return (
    <>
      <Title page={t("captcha.title")} route="/admin/captcha" />
      <div class="flex-1 flex flex-col items-center p-3 lg:p-6">
        <Form onSubmit={onSubmit} class="w-full max-w-5xl flex flex-col space-y-2">
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--settings-20-regular] w-5 h-5" />
            <span>{t("captcha.title")}</span>
          </h3>
          <div class="flex flex-col space-y-2 lg:flex-row lg:space-y-0 lg:space-x-2 lg:items-end">
            <Field name="enabled" type="boolean">
              {(field, props) => (
                <Checkbox
                  class="flex-1"
                  inputProps={props}
                  checked={field.value}
                  error={field.error}
                  title={t("captcha.form.enabled.label")}
                >
                  <span class="flex-1 text-start">{t("captcha.form.enabled.label")}</span>
                </Checkbox>
              )}
            </Field>
            <Field name="validator" validate={[required(t("captcha.form.validator.required"))]}>
              {(field, props) => (
                <Select
                  name={field.name}
                  label={t("captcha.form.validator.label")}
                  disabled={getValue(form, "enabled") === false}
                  class="flex-1"
                  error={field.error}
                  placeholder={t("captcha.form.validator.placeholder")}
                  items={[
                    {
                      value: "pow",
                      label: t("captcha.form.validator.type.pow"),
                      icon: "icon-[fluent--code-20-regular]",
                    },
                    {
                      value: "image",
                      label: t("captcha.form.validator.type.image"),
                      icon: "icon-[fluent--image-20-regular]",
                    },
                  ]}
                  value={field.value ? [field.value as string] : undefined}
                  inputProps={props}
                />
              )}
            </Field>
          </div>
          <Field name="difficulty" type="number">
            {(field, props) => (
              <Slider
                disabled={getValue(form, "enabled") === false}
                class="flex-1"
                label={t("captcha.form.difficulty.label")}
                max={10}
                min={1}
                step={1}
                name={field.name}
                value={[field.value || 1]}
                inputProps={props}
                onValueChange={(e) => {
                  setValues(form, { [field.name]: e.value[0] });
                }}
              />
            )}
          </Field>
          <Button
            type="submit"
            level="primary"
            class="!mt-4"
            loading={config.isLoading || mutation.isPending}
            disabled={config.isLoading || mutation.isPending}
          >
            {t("general.actions.save.title")}
          </Button>
        </Form>
      </div>
    </>
  );
}
