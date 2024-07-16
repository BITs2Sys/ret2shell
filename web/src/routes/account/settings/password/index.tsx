import { changePassword } from "@/lib/api/account";
import { resetUser } from "@/lib/storage/account";
import { t } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import Button from "@/lib/widgets/button";
import Input from "@/lib/widgets/input";
import { createForm, custom, getValue, minLength, pattern, required } from "@modular-forms/solid";
import { useNavigate } from "@solidjs/router";
import type { HTTPError } from "ky";
import { createSignal } from "solid-js";

type ChangePasswordForm = {
  old_password: string;
  new_password: string;
  confirm_password: string;
};

export default function () {
  const [form, { Form, Field }] = createForm<ChangePasswordForm>();
  const [loading, setLoading] = createSignal(false);
  const navigate = useNavigate();
  function onSubmit(result: ChangePasswordForm) {
    setLoading(true);
    changePassword({
      old_password: result.old_password,
      new_password: result.new_password,
    })
      .then(() => {
        addToast({
          level: "success",
          description: t("account.settings.password.success")!,
          duration: 5000,
        });
      })
      .catch((e: HTTPError) => {
        e.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.saveFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setLoading(false);
        setTimeout(() => {
          resetUser();
          navigate("/account/login");
        }, 1000);
      });
  }
  return (
    <div class="flex flex-col p-3 lg:p-6 w-full items-center">
      <Form onSubmit={onSubmit} class="flex flex-col w-full max-w-5xl space-y-2 relative">
        <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
          <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
          <span>{t("account.settings.password.title")}</span>
        </h3>
        <Field name="old_password" validate={[required(t("account.settings.password.oldPasswordRequired")!)]}>
          {(field, props) => (
            <Input
              icon={<span class="icon-[fluent--password-20-regular] w-5 h-5" />}
              title={t("account.settings.password.oldPassword")}
              placeholder={t("account.settings.password.oldPassword")}
              {...props}
              value={field.value}
              error={field.error}
              required
              type="password"
            />
          )}
        </Field>
        <Field
          name="new_password"
          validate={[
            required(t("account.settings.password.newPasswordRequired")!),
            minLength(8, t("account.register.passwordMinLength")!),
            pattern(
              // biome-ignore lint/correctness/noEmptyCharacterClassInRegex: password allows any characters
              /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[^]{8,40}$/,
              t("account.register.passwordTooWeak")!
            ),
          ]}
        >
          {(field, props) => (
            <Input
              icon={<span class="icon-[fluent--password-20-regular] w-5 h-5" />}
              title={t("account.settings.password.newPassword")}
              placeholder={t("account.settings.password.newPassword")}
              {...props}
              value={field.value}
              error={field.error}
              required
              type="password"
            />
          )}
        </Field>
        <Field
          name="confirm_password"
          validate={[
            required(t("account.settings.password.confirmPasswordRequired")!),
            custom((v) => {
              if (v !== getValue(form, "new_password")) {
                return false;
              }
              return true;
            }, t("account.settings.password.confirmPasswordMismatch")!),
          ]}
        >
          {(field, props) => (
            <Input
              icon={<span class="icon-[fluent--password-20-regular] w-5 h-5" />}
              title={t("account.settings.password.confirmPassword")}
              placeholder={t("account.settings.password.confirmPassword")}
              {...props}
              value={field.value}
              error={field.error}
              required
              type="password"
            />
          )}
        </Field>
        <Button type="submit" level="primary" class="!mt-4" loading={loading()} disabled={loading()}>
          {t("form.save")}
        </Button>
      </Form>
    </div>
  );
}
