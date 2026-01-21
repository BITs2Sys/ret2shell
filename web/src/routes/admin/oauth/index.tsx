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
  }

  const createInstituteMutation = useCreateInstituteMutation({ onSuccess });
  const updateInstituteMutation = useUpdateInstituteMutation({ onSuccess });
  const deleteInstituteMutation = useDeleteInstituteMutation({ onSuccess });

  const createOAuthProviderMutation = useCreateOAuthProviderMutation({ onSuccess });
  const updateOAuthProviderMutation = useUpdateOAuthProviderMutation({ onSuccess });
  const deleteOAuthProviderMutation = useDeleteOAuthProviderMutation({ onSuccess });

  const [providerCreationFormOpen, setProviderCreationFormOpen] = createSignal(false);
  const [providerFormOpen, setProviderFormOpen] = createSignal(false);
  const [instituteCreationFormOpen, setInstituteCreationFormOpen] = createSignal(false);
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
              open={providerCreationFormOpen()}
              onOpenChange={(detail) => setProviderCreationFormOpen(detail.open)}
            >
              <ProviderForm
                onDone={async (v) => {
                  await createOAuthProviderMutation.mutateAsync(v);
                  setProviderCreationFormOpen(false);
                }}
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
                  open={providerFormOpen()}
                  onOpenChange={(detail) => setProviderFormOpen(detail.open)}
                >
                  <ProviderForm
                    provider={service.provider}
                    onDone={async (v) => {
                      await updateOAuthProviderMutation.mutateAsync({ service: service.provider, req: v });
                      setProviderFormOpen(false);
                    }}
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
              onOpenChange={(detail) => setInstituteCreationFormOpen(detail.open)}
              open={instituteCreationFormOpen()}
            >
              <InstituteForm
                onDone={async (v) => {
                  await createInstituteMutation.mutateAsync(v);
                  setInstituteCreationFormOpen(false);
                }}
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
                  open={instituteFormOpen()}
                  onOpenChange={(detail) => setInstituteFormOpen(detail.open)}
                >
                  <InstituteForm
                    editSource={institute}
                    onDone={async (v) => {
                      await updateInstituteMutation.mutateAsync(v);
                      setInstituteFormOpen(false);
                    }}
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
