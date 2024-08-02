import { getUserList } from "@api/user";
import { mediaPath } from "@lib/utils/media";
import { type User, permissionToIcon } from "@models/user";
import { accountStore, refreshInstitutes } from "@storage/account";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Avatar from "@widgets/avatar";
import Dialog from "@widgets/dialog";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Pagination from "@widgets/pagination";
import Select from "@widgets/select";
import Tag from "@widgets/tag";
import type { HTTPError } from "ky";
import { For, Show, createEffect, createMemo, createSignal, untrack } from "solid-js";

type OrderType = "id" | "account" | "institute_id" | "registered_at";

export default function () {
  const [users, setUsers] = createSignal([] as User[]);
  const [page, setPage] = createSignal(1);
  const pageSize = 15;
  const [loading, setLoading] = createSignal(true);
  const [total, setTotal] = createSignal(0);
  const [filter, setFilter] = createSignal(null as string | null);
  const [order, setOrder] = createSignal("id" as "id" | "account" | "institute_id" | "registered_at");
  const [instituteId, setInstituteId] = createSignal(null as number | null);
  function refreshUsers() {
    setLoading(true);
    getUserList(page(), pageSize, order() || "id", filter() ?? undefined, instituteId() ?? undefined)
      .then((resp) => {
        setUsers(resp[0]);
        setTotal(resp[1]);
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("admin.users.fetchFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => setLoading(false));
  }

  const institutesSelect = createMemo(() => {
    return accountStore.institutes.map((i) => ({
      value: i.id.toString(),
      label: i.name,
      icon: "icon-[fluent--hat-graduation-20-regular] w-5 h-5",
    }));
  });
  refreshInstitutes();
  createEffect(() => {
    if (page()) {
      untrack(refreshUsers);
    }
  });
  return (
    <div class="flex-1 flex flex-col items-center">
      <Title title={`${t("admin.users.title")} - ${platformStore.config.name || t("platform.name")}`} />
      <div class="w-full p-3 lg:p-6 flex flex-col flex-1">
        <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
          <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
          <span class="flex-1 text-start">{t("admin.users.title")}</span>
          <Select
            class="flex-1 max-w-48 min-w-0"
            size="sm"
            placeholder={t("admin.users.sortBy")}
            items={[
              {
                value: "id",
                label: "ID",
                icon: "icon-[fluent--number-symbol-24-regular] w-5 h-5",
              },
              {
                value: "account",
                label: t("admin.users.account")!,
                icon: "icon-[fluent--number-symbol-24-regular] w-5 h-5",
              },
              {
                value: "institute_id",
                label: t("admin.users.institute")!,
                icon: "icon-[fluent--number-symbol-24-regular] w-5 h-5",
              },
              {
                value: "registered_at",
                label: t("admin.users.registeredAt")!,
                icon: "icon-[fluent--number-symbol-24-regular] w-5 h-5",
              },
            ]}
            onValueChange={(v) => {
              setOrder((v.value.at(0) || "id") as OrderType);
              refreshUsers();
            }}
            value={instituteId() ? [instituteId()!.toString()] : undefined}
          />
          <Select
            class="flex-1 max-w-64 min-w-0"
            size="sm"
            placeholder={t("admin.users.selectInstitute")}
            items={institutesSelect()}
            onValueChange={(v) => {
              setInstituteId((v.value.at(0) && Number.parseInt(v.value.at(0)!)) || null);
              refreshUsers();
            }}
            value={instituteId() ? [instituteId()!.toString()] : undefined}
          />
          <Input
            class="w-80"
            size="sm"
            icon={<span class="icon-[fluent--filter-16-regular] w-5 h-5" />}
            placeholder={t("admin.users.filterPlaceholder")}
            onChange={(e) => {
              setFilter(e.currentTarget.value);
              refreshUsers();
            }}
          />
        </h3>
        <Show when={loading()}>
          <div class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-4 px-2 hover:bg-layer-content/5">
            <LoadingTips />
          </div>
        </Show>
        <div class="flex-1 flex flex-col">
          <For each={users()}>
            {(user) => (
              <div class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-4 px-2">
                <Avatar
                  class="w-6 h-6"
                  src={(user.avatar && mediaPath(user.avatar)) || undefined}
                  fallback={user.account || undefined}
                />
                <span class="flex-1 truncate text-start">
                  <span>{user.nickname}</span>
                  <span class="font-normal px-2 opacity-60">
                    {user.account}#{user.id.toString(16).padStart(6, "0")}
                  </span>
                </span>
                <For each={user.permissions}>{(permission) => <span class={permissionToIcon(permission)} />}</For>
                <Show when={user.institute_id}>
                  <Tag level="info">
                    <span>{accountStore.institutes.find((v) => v.id === user.institute_id)?.name}</span>
                  </Tag>
                </Show>
                <span class="font-normal">{user.registered_at.toFormat("yyyy-MM-dd HH:mm:ss")}</span>
                <Dialog
                  size="sm"
                  ghost
                  square
                  btnContent={<span class="icon-[fluent--edit-20-regular] w-5 h-5 text-info" />}
                  title={t("form.edit")}
                >
                  {null}
                </Dialog>
              </div>
            )}
          </For>
        </div>
        <Pagination
          class="p-6 lg:p-9"
          count={total()}
          pageSize={pageSize}
          page={page()}
          onPageChange={(page) => setPage(page.page)}
        />
      </div>
    </div>
  );
}
