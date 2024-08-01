import type { Institute } from "@models/institute";
import { createForm, required, setValue, setValues } from "@modular-forms/solid";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Input from "@widgets/input";
import Select from "@widgets/select";
import { createEffect, untrack } from "solid-js";

type Providers = "xdu" | "xmu" | "jiangnan" | "nwnu" | "taru";

type FormType = {
  name: string;
  provider?: Providers;
};

export default function InstituteForm(props: {
  onDone?: (result: Institute) => void;
  editSource?: Institute;
  loading?: boolean;
}) {
  const [form, { Form, Field }] = createForm<FormType>();
  createEffect(() => {
    if (props.editSource) {
      untrack(() => {
        setValues(form, {
          name: props.editSource!.name,
          provider: (props.editSource?.provider as Providers) || undefined,
        });
      });
    }
  });
  function onSubmit(result: FormType) {
    props.onDone?.({
      id: props.editSource?.id || 0,
      name: result.name,
      description: null,
      logo: null,
      provider: result.provider || null,
    });
  }
  return (
    <Form onSubmit={onSubmit} class="flex flex-col w-96 space-y-2 relative">
      <Field name="name" validate={[required(t("admin.institute.nameRequired")!)]}>
        {(field, props) => (
          <Input
            icon={<span class="icon-[fluent--flag-20-regular] w-5 h-5" />}
            title={t("admin.institute.name")}
            placeholder={t("admin.institute.name")}
            {...props}
            value={field.value}
            error={field.error}
            required
          />
        )}
      </Field>
      <Field name="provider">
        {(field, props) => (
          <Select
            name={field.name}
            label={t("admin.institute.provider")!}
            class="flex-1"
            error={field.error}
            placeholder={t("admin.institute.providerNeeded")}
            items={[
              {
                value: "xdu",
                label: t("account.oauth.xdu.title")!,
                icon: "icon-[fluent--hat-graduation-20-regular]",
              },
              {
                value: "xmu",
                label: t("account.oauth.xmu.title")!,
                icon: "icon-[fluent--hat-graduation-20-regular]",
              },
              {
                value: "jiangnan",
                label: t("account.oauth.jiangnan.title")!,
                icon: "icon-[fluent--hat-graduation-20-regular]",
              },
              {
                value: "nwnu",
                label: t("account.oauth.nwnu.title")!,
                icon: "icon-[fluent--hat-graduation-20-regular]",
              },
              {
                value: "taru",
                label: t("account.oauth.taru.title")!,
                icon: "icon-[fluent--hat-graduation-20-regular]",
              },
            ]}
            value={field.value ? [field.value as string] : undefined}
            inputProps={props}
            onValueChange={(e) => {
              setValue(form, "provider", e.value.at(0) as Providers);
            }}
          >
            {/* TODO: integrate with modular-forms */}
          </Select>
        )}
      </Field>
      <Button type="submit" level="primary" class="!mt-4" loading={props.loading} disabled={props.loading}>
        {props.editSource ? t("form.save") : t("form.create")}
      </Button>
    </Form>
  );
}
