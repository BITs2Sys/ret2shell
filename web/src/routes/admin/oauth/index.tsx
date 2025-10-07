import {
  useCreateInstituteMutation,
  useCreateOAuthProviderMutation,
  useDeleteInstituteMutation,
  useDeleteOAuthProviderMutation,
  useInstitutes,
  useOAuthProviders,
  useUpdateInstituteMutation,
  useUpdateOAuthProviderMutation,
} from "@api/account";
import { mediaPath } from "@lib/utils/media";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Avatar from "@widgets/avatar";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Dialog from "@widgets/dialog";
import Popover from "@widgets/popover";
import { createSignal, For, Show } from "solid-js";
import InstituteForm from "./_blocks/institute-form";
import ProviderForm from "./_blocks/provider-form";

export default function () {
  const oauthProviders = useOAuthProviders();
  const institutes = useInstitutes();

  function onSuccess() {
    oauthProviders.refetch();
    institutes.refetch();
    addToast({
      level: "success",
      description: t("general.actions.create.status.success"),
      duration: 5000,
    });
  }

  const createInstituteMutation = useCreateInstituteMutation({ onSuccess });
  const updateInstituteMutation = useUpdateInstituteMutation({ onSuccess });
  const deleteInstituteMutation = useDeleteInstituteMutation({ onSuccess });

  const createOAuthProviderMutation = useCreateOAuthProviderMutation({ onSuccess });
  const updateOAuthProviderMutation = useUpdateOAuthProviderMutation({ onSuccess });
  const deleteOAuthProviderMutation = useDeleteOAuthProviderMutation({ onSuccess });

  const [providerFormOpen, setProviderFormOpen] = createSignal(false);
  const [instituteFormOpen, setInstituteFormOpen] = createSignal(false);
  return (
    <>
      <Title page={t("oauth.title")} route="/admin/oauth" />
      <div class="flex-1 flex flex-col items-center p-3 lg:p-6">
        <div class="w-full max-w-5xl flex flex-col">
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--settings-20-regular] w-5 h-5" />
            <span class="flex-1 text-start">{t("oauth.title")}</span>
            <Dialog
              level="primary"
              size="sm"
              title={t("general.actions.create.title")}
              btnContent={
                <>
                  <span class="shrink-0 icon-[fluent--add-20-regular] w-5 h-5" />
                  <span>{t("general.actions.create.title")}</span>
                </>
              }
              // onClick={() => setProviderFormOpen(true)}
              open={providerFormOpen()}
              onOpenChange={(detail) => setProviderFormOpen(detail.open)}
            >
              <ProviderForm
                onDone={(v) => Promise.resolve(createOAuthProviderMutation.mutate(v))}
                loading={createOAuthProviderMutation.isPending}
              />
            </Dialog>
          </h3>
          <For each={oauthProviders.data}>
            {(service) => (
              <div class="h-12 flex items-center border-b border-b-layer-content/10 space-x-2">
                <Avatar src={(service.avatar && mediaPath(service.avatar)) ?? undefined} class="w-5 h-5" />
                <h4 class="font-bold text-start flex-1">
                  <span>{service.name}</span>
                </h4>
                <span class="text-success">{t("oauth.configured")}</span>
                <Dialog
                  ghost
                  size="sm"
                  square
                  title={t("general.actions.edit.title")}
                  btnContent={<span class="shrink-0 icon-[fluent--edit-20-regular] w-5 h-5" />}
                >
                  <ProviderForm
                    editSource={service}
                    onDone={(v) =>
                      Promise.resolve(updateOAuthProviderMutation.mutate({ service: service.provider, req: v }))
                    }
                    loading={updateOAuthProviderMutation.isPending}
                  />
                </Dialog>
                <Popover
                  size="sm"
                  ghost
                  square
                  title={t("general.actions.delete.title")}
                  btnContent={<span class="shrink-0 icon-[fluent--delete-20-regular] w-5 h-5" />}
                >
                  <Card contentClass="p-2 flex flex-row space-x-2 items-center">
                    <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5 text-error" />
                    <span>{t("general.actions.delete.message")}</span>
                    <Button
                      level="error"
                      size="sm"
                      title={t("general.actions.confirm.title")}
                      onClick={() => deleteOAuthProviderMutation.mutate({ service: service.provider })}
                      loading={deleteOAuthProviderMutation.isPending}
                    >
                      <span>{t("general.actions.confirm.title")}</span>
                    </Button>
                  </Card>
                </Popover>
              </div>
            )}
          </For>
          <div class="h-36" />
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--hat-graduation-20-regular] w-5 h-5" />
            <span class="flex-1 text-start">{t("institute.title")}</span>
            <Dialog
              level="primary"
              size="sm"
              title={t("general.actions.create.title")}
              btnContent={
                <>
                  <span class="shrink-0 icon-[fluent--add-20-regular] w-5 h-5" />
                  <span>{t("general.actions.create.title")}</span>
                </>
              }
              // onClick={() => setInstituteFormOpen(true)}
              onOpenChange={(detail) => setInstituteFormOpen(detail.open)}
              open={instituteFormOpen()}
            >
              <InstituteForm
                onDone={(v) => Promise.resolve(createInstituteMutation.mutate(v))}
                loading={createInstituteMutation.isPending}
              />
            </Dialog>
          </h3>
          <For each={institutes.data}>
            {(institute) => (
              <div class="h-12 flex items-center border-b border-b-layer-content/10 space-x-2">
                <span class="shrink-0 icon-[fluent--hat-graduation-20-regular] w-5 h-5" />
                <span class="flex-1 text-start">{institute.name}</span>
                <Show
                  when={institute.provider}
                  fallback={<span class="text-warning px-2">{t("institute.manual")}</span>}
                >
                  <span class="text-success px-2">
                    {t("institute.withOAuth")}: {institute.provider}
                  </span>
                </Show>
                <Dialog
                  ghost
                  size="sm"
                  square
                  title={t("general.actions.edit.title")}
                  btnContent={<span class="shrink-0 icon-[fluent--edit-20-regular] w-5 h-5" />}
                >
                  <InstituteForm
                    editSource={institute}
                    onDone={(v) => Promise.resolve(updateInstituteMutation.mutate(v))}
                    loading={updateInstituteMutation.isPending}
                  />
                </Dialog>
                <Popover
                  size="sm"
                  ghost
                  square
                  title={t("general.actions.delete.title")}
                  btnContent={<span class="shrink-0 icon-[fluent--delete-20-regular] w-5 h-5" />}
                >
                  <Card contentClass="p-2 flex flex-row space-x-2 items-center">
                    <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5 text-error" />
                    <span>{t("general.actions.delete.message")}</span>
                    <Button
                      level="error"
                      size="sm"
                      title={t("general.actions.confirm.title")}
                      onClick={() => deleteInstituteMutation.mutate({ id: institute.id })}
                      loading={deleteInstituteMutation.isPending}
                    >
                      <span>{t("general.actions.confirm.title")}</span>
                    </Button>
                  </Card>
                </Popover>
              </div>
            )}
          </For>
        </div>
      </div>
    </>
  );
}
