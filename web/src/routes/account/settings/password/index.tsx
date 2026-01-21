import { useChangePasswordMutation } from "@api/account";
import { createForm, custom, getValue, minLength, pattern, required } from "@modular-forms/solid";
import { useNavigate } from "@solidjs/router";
import { accountStore, resetUser } from "@storage/account";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Input from "@widgets/input";

type ChangePasswordForm = {
  old_password: string;
  new_password: string;
  confirm_password: string;
};

export default function () {
  const [form, { Form, Field }] = createForm<ChangePasswordForm>();
  const navigate = useNavigate();

  const mutation = useChangePasswordMutation({
    onSuccess: () => {
      setTimeout(() => {
        resetUser();
        navigate("/account/login");
      }, 1000);
    },
  });

  async function onSubmit(result: ChangePasswordForm) {
    mutation.mutate(result);
  }
  return (
    <>
      <Title page={t("account.password.title")} route="/account/settings/password" />
      <div class="flex flex-col p-3 lg:p-6 w-full items-center">
        <Form onSubmit={onSubmit} class="flex flex-col w-full max-w-5xl space-y-2 relative">
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--settings-20-regular] w-5 h-5" />
            <span>{t("account.password.title")}</span>
          </h3>
          <input
            class="hidden"
            type="text"
            name="username"
            value={accountStore.account!}
            autocomplete="username"
            disabled
          />
          <Field name="old_password" validate={[required(t("account.form.oldPassword.required"))]}>
            {(field, props) => (
              <Input
                icon={<span class="shrink-0 icon-[fluent--password-20-regular] w-5 h-5" />}
                title={t("account.form.oldPassword.label")}
                placeholder={t("account.form.oldPassword.placeholder")}
                {...props}
                value={field.value}
                error={field.error}
                required
                type="password"
                autocomplete="current-password"
              />
            )}
          </Field>
          <Field
            name="new_password"
            validate={[
              required(t("account.form.password.required")),
              minLength(8, t("account.form.password.minimumLength")),
              pattern(
                // biome-ignore lint/correctness/noEmptyCharacterClassInRegex: password allows any characters
                /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[^]{8,40}$/,
                t("account.form.password.tooWeak")
              ),
            ]}
          >
            {(field, props) => (
              <Input
                icon={<span class="shrink-0 icon-[fluent--password-20-regular] w-5 h-5" />}
                title={t("account.form.password.label")}
                placeholder={t("account.form.password.placeholder")}
                {...props}
                value={field.value}
                error={field.error}
                required
                autocomplete="new-password"
                type="password"
              />
            )}
          </Field>
          <Field
            name="confirm_password"
            validate={[
              required(t("account.form.password.confirmRequired")),
              custom((v) => {
                if (v !== getValue(form, "new_password")) {
                  return false;
                }
                return true;
              }, t("account.form.password.confirmMismatch")),
            ]}
          >
            {(field, props) => (
              <Input
                icon={<span class="shrink-0 icon-[fluent--password-20-regular] w-5 h-5" />}
                title={t("account.form.password.confirmLabel")}
                placeholder={t("account.form.password.confirmPlaceholder")}
                {...props}
                value={field.value}
                error={field.error}
                required
                autocomplete="new-password"
                type="password"
              />
            )}
          </Field>
          <Button
            type="submit"
            level="primary"
            class="mt-4!"
            loading={mutation.isPending}
            disabled={mutation.isPending}
          >
            {t("general.actions.save.title")}
          </Button>
        </Form>
      </div>
    </>
  );
}
