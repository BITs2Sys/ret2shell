import { getPlatformConfig, updatePlatformConfig } from "@/lib/api/platform";
import type { CaptchaConfig, Config } from "@/lib/models/config";
import { addToast } from "@/lib/storage/toast";
import Button from "@/lib/widgets/button";
import Checkbox from "@/lib/widgets/checkbox";
import Select from "@/lib/widgets/select";
import Slider from "@/lib/widgets/slider";
import { createForm, getValue, required, setValue, setValues } from "@modular-forms/solid";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import type { HTTPError } from "ky";
import { createSignal, onMount } from "solid-js";

export default function () {
  const [form, { Form, Field }] = createForm<CaptchaConfig>();
  const [loading, setLoading] = createSignal(false);
  const [config, setConfig] = createSignal(null as null | Config);
  onMount(() => {
    getPlatformConfig().then((resp) => {
      setConfig(resp);
      setValues(form, {
        enabled: resp.captcha.enabled,
        difficulty: resp.captcha.difficulty,
        validator: resp.captcha.validator,
      });
    });
  });
  function onSubmit(result: CaptchaConfig) {
    setLoading(true);
    if (!config()) {
      addToast({
        level: "error",
        description: t("admin.platform.fetchNotReady")!,
        duration: 5000,
      });
      return;
    }
    const mergedConfig = {
      ...config(),
      captcha: {
        enabled: result.enabled,
        difficulty: result.difficulty,
        validator: result.validator,
      },
    } as Config;
    updatePlatformConfig(mergedConfig)
      .then(() => {
        setConfig(mergedConfig);
        addToast({
          level: "success",
          description: t("admin.platform.updateSuccess")!,
          duration: 5000,
        });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("admin.platform.updateFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => setLoading(false));
  }
  return (
    <>
      <Title title={`${t("admin.captcha.title")} - ${platformStore.config.name || t("platform.name")}`} />
      <div class="flex-1 flex flex-col items-center">
        <div class="w-full max-w-5xl p-3 lg:p-6 flex flex-col space-y-2">
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
            <span>{t("admin.captcha.title")}</span>
          </h3>
          <Form onSubmit={onSubmit} class="w-full flex flex-col space-y-2">
            <div class="flex flex-col space-y-2 lg:flex-row lg:space-y-0 lg:space-x-2 lg:items-end">
              <Field name="enabled" type="boolean">
                {(field, props) => (
                  <Checkbox
                    class="flex-1"
                    inputProps={props}
                    checked={field.value}
                    error={field.error}
                    title={t("captcha.enabled")}
                  >
                    <span class="flex-1 text-start">{t("captcha.enabled")}</span>
                  </Checkbox>
                )}
              </Field>
              <Field name="validator" validate={[required(t("admin.captcha.validatorRequired")!)]}>
                {(field, props) => (
                  <Select
                    label={t("admin.captcha.validator")!}
                    disabled={getValue(form, "enabled") === false}
                    class="flex-1"
                    error={field.error}
                    placeholder={t("admin.captcha.select")}
                    name={props.name}
                    items={[
                      {
                        value: "pow",
                        label: t("admin.captcha.pow")!,
                        icon: "icon-[fluent--code-20-regular]",
                      },
                      {
                        value: "image",
                        label: t("admin.captcha.image")!,
                        icon: "icon-[fluent--image-20-regular]",
                      },
                    ]}
                    value={field.value ? [field.value as string] : undefined}
                    onValueChange={(v) => {
                      setValue(form, "validator", v.value[0] as "pow" | "image");
                    }}
                  />
                )}
              </Field>
            </div>
            <Field name="difficulty" type="number">
              {(field, props) => (
                <Slider
                  disabled={getValue(form, "enabled") === false}
                  class="flex-1"
                  label={t("admin.captcha.difficulty")!}
                  name={props.name}
                  max={10}
                  min={1}
                  step={1}
                  value={[field.value ?? 1]}
                  onValueChange={(v) => {
                    setValue(form, "difficulty", v.value[0]);
                  }}
                />
              )}
            </Field>
            <Button type="submit" level="primary" class="!mt-4" loading={loading()} disabled={!config() || loading()}>
              {t("form.save")}
            </Button>
          </Form>
        </div>
      </div>
    </>
  );
}
