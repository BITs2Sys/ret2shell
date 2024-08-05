import { getGameAdmins, updateGame, updateGameAdmins } from "@api/game";
import { getUserList } from "@api/user";
import { Popover as ArkPopover } from "@ark-ui/solid";
import { mediaPath } from "@lib/utils/media";
import { Permission, type User, permissionToIcon } from "@models/user";
import { A } from "@solidjs/router";
import { accountStore, refreshInstitutes } from "@storage/account";
import { gameStore, setGameStore } from "@storage/game";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Avatar from "@widgets/avatar";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import Dialog from "@widgets/dialog";
import Divider from "@widgets/divider";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Popover from "@widgets/popover";
import Tag from "@widgets/tag";
import type { HTTPError } from "ky";
import { For, Show, createEffect, createSignal, untrack } from "solid-js";

function InstituteManagement() {
  const [loading, setLoading] = createSignal(true);
  refreshInstitutes().then(() => setLoading(false));
  function handleChangePolicy(restrict: boolean) {
    if (gameStore.current) {
      setLoading(true);
      updateGame(gameStore.current.id, {
        ...gameStore.current,
        access_policy: {
          ...gameStore.current.access_policy,
          restrict,
        },
      })
        .then((resp) => {
          addToast({
            level: "success",
            description: t("form.saveSuccess")!,
            duration: 5000,
          });
          setGameStore({ current: resp });
        })
        .catch((err: HTTPError) => {
          err.response.text().then((text) => {
            addToast({
              level: "error",
              description: `${t("form.saveFailed")}: ${text}`,
              duration: 5000,
            });
          });
        })
        .finally(() => setLoading(false));
    }
  }
  function handleChangeInstitute(institute: number, enabled: boolean) {
    if (gameStore.current) {
      setLoading(true);
      const institutes = JSON.parse(JSON.stringify(gameStore.current.access_policy.institutes));
      if (enabled) {
        institutes.push(institute);
      } else {
        institutes.splice(institutes.indexOf(institute), 1);
      }
      updateGame(gameStore.current.id, {
        ...gameStore.current,
        access_policy: {
          ...gameStore.current.access_policy,
          institutes,
        },
      })
        .then((resp) => {
          addToast({
            level: "success",
            description: t("form.saveSuccess")!,
            duration: 5000,
          });
          setGameStore({ current: resp });
        })
        .catch((err: HTTPError) => {
          err.response.text().then((text) => {
            addToast({
              level: "error",
              description: `${t("form.saveFailed")}: ${text}`,
              duration: 5000,
            });
          });
        })
        .finally(() => setLoading(false));
    }
  }
  return (
    <>
      <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
        <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
        <span>{t("game.admin.organize.title")}</span>
      </h3>
      <Show when={loading()}>
        <div class="h-12 flex flex-row items-center border-b border-b-layer-content/10">
          <LoadingTips />
        </div>
      </Show>
      <Checkbox
        checked={gameStore.current?.access_policy.restrict}
        title={t("game.admin.organize.restrict")}
        onChange={() => handleChangePolicy(!gameStore.current?.access_policy.restrict)}
      >
        <span class="flex-1 text-start">{t("game.admin.organize.restrict")}</span>
      </Checkbox>
      <div class="flex flex-col space-y-1">
        <label class="label">{t("game.admin.organize.instituteEnabled")}</label>
        <div class="flex flex-row flex-wrap">
          <For each={accountStore.institutes}>
            {(institute) => (
              <Checkbox
                class="m-1 flex-none"
                checked={gameStore.current?.access_policy.institutes.includes(institute.id)}
                onChange={() => {
                  handleChangeInstitute(
                    institute.id,
                    !gameStore.current?.access_policy.institutes.includes(institute.id)
                  );
                }}
              >
                <span class="flex-1 text-start">{institute.name}</span>
              </Checkbox>
            )}
          </For>
        </div>
      </div>
    </>
  );
}
function AdministratorsManagement() {
  const [loading, setLoading] = createSignal(false);
  const [admins, setAdmins] = createSignal([] as User[]);
  createEffect(() => {
    if (gameStore.current?.admins) {
      untrack(() => {
        setLoading(true);
        getGameAdmins(gameStore.current!.id)
          .then((resp) => {
            setAdmins(resp);
          })
          .catch((err: HTTPError) => {
            err.response.text().then((text) => {
              addToast({
                level: "error",
                description: `${t("game.admin.administrators.fetchFailed")}: ${text}`,
                duration: 5000,
              });
            });
          })
          .finally(() => {
            setLoading(false);
          });
      });
    }
  });

  const [adminSearch, setAdminSearch] = createSignal<string>("");
  const [searching, setSearching] = createSignal(false);
  const [searchedUsers, setSearchedUsers] = createSignal([] as User[]);
  createEffect(() => {
    if (adminSearch()) {
      untrack(() => {
        setSearching(true);
        setSearchedUsers([]);
        getUserList(1, 10, "id", adminSearch())
          .then((resp) => {
            setSearchedUsers(resp[0]);
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
          .finally(() => {
            setSearching(false);
          });
      });
    }
  });
  const [adding, setAdding] = createSignal(false);
  function handleAddAdmin(user: User) {
    setAdding(true);
    updateGameAdmins(gameStore.current!.id, [...gameStore.current!.admins, user.id])
      .then((resp) => {
        setGameStore({ current: resp });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.admin.administrators.addFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setAdding(false);
        setSearching(false);
        setAdminSearch("");
        setSearchedUsers([]);
      });
  }
  function handleDeleteAdmin(user: User) {
    setLoading(true);
    updateGameAdmins(
      gameStore.current!.id,
      gameStore.current!.admins.filter((v) => v !== user.id)
    )
      .then((resp) => {
        setGameStore({ current: resp });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.admin.administrators.deleteFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setLoading(false);
      });
  }
  return (
    <>
      <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
        <span class="icon-[fluent--person-key-20-regular] w-5 h-5" />
        <span>{t("game.admin.administrators.title")}</span>
      </h3>
      <Show when={loading()}>
        <div class="h-12 flex flex-row items-center border-b border-b-layer-content/10">
          <LoadingTips />
        </div>
      </Show>
      <ArkPopover.Root autoFocus={false} open={!!adminSearch()} closeOnInteractOutside={false}>
        <ArkPopover.Anchor>
          <Input
            placeholder={t("game.admin.administrators.addPlaceholder")}
            title={t("game.admin.administrators.add")}
            icon={<span class="icon-[fluent--person-key-20-regular] w-5 h-5" />}
            onChange={(e) => setAdminSearch(e.currentTarget.value)}
          />
        </ArkPopover.Anchor>
        <ArkPopover.Positioner class="w-full">
          <ArkPopover.Content class="popover card w-full z-50">
            <div class="card-content p-2 flex flex-col space-y-2">
              <Show when={searching()}>
                <LoadingTips />
              </Show>
              <For
                each={searchedUsers()}
                fallback={
                  <Show when={!searching() && adminSearch()}>
                    <div class="h-12 flex items-center font-bold space-x-4 px-2">
                      <span class="icon-[fluent--emoji-sad-slight-20-regular] w-5 h-5" />
                      <span class="font-bold opacity-60">{t("game.admin.administrators.noAdmins")}</span>
                    </div>
                  </Show>
                }
              >
                {(user) => (
                  <Dialog
                    disabled={
                      !user.permissions.includes(Permission.Game) || gameStore.current?.admins.includes(user.id)
                    }
                    ghost
                    btnContent={
                      <>
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
                        <Show when={!user.permissions.includes(Permission.Game)}>
                          <Tag level="error">
                            <span>{t("game.admin.administrators.noPermission")}</span>
                          </Tag>
                        </Show>
                        <Show when={gameStore.current?.admins.includes(user.id)}>
                          <Tag level="success">
                            <span>{t("game.admin.administrators.alreadyAdded")}</span>
                          </Tag>
                        </Show>
                      </>
                    }
                  >
                    <div class="flex flex-col w-64 space-y-2 items-center">
                      <div class="flex flex-row space-x-4 justify-start items-center w-full">
                        <Avatar
                          class="w-12 h-12"
                          src={(user.avatar && mediaPath(user.avatar)) || undefined}
                          fallback={user.account || undefined}
                        />
                        <div class="flex flex-col space-x-0 items-start justify-center">
                          <h2 class="font-bold text-lg">{user.nickname}</h2>
                          <p class="font-normal opacity-60">
                            {user.account}#{user.id.toString(16).padStart(6, "0")}
                          </p>
                        </div>
                      </div>
                      <Divider class="w-full" />
                      <p>{t("game.admin.administrators.confirm")}</p>
                      <Button
                        level="info"
                        class="w-full"
                        onClick={() => handleAddAdmin(user)}
                        loading={adding()}
                        disabled={adding()}
                      >
                        {t("form.confirm")}
                      </Button>
                    </div>
                  </Dialog>
                )}
              </For>
            </div>
          </ArkPopover.Content>
        </ArkPopover.Positioner>
      </ArkPopover.Root>
      <div class="flex-1 flex flex-col">
        <For each={admins()}>
          {(user) => (
            <div class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-4 px-2">
              <Avatar
                class="w-6 h-6"
                src={(user.avatar && mediaPath(user.avatar)) || undefined}
                fallback={user.account || undefined}
              />
              <A class="flex-1 truncate text-start hover:underline" href={`/users/${user.id}`}>
                <span>{user.nickname}</span>
                <span class="font-normal px-2 opacity-60">
                  {user.account}#{user.id.toString(16).padStart(6, "0")}
                </span>
              </A>
              <For each={user.permissions}>{(permission) => <span class={permissionToIcon(permission)} />}</For>
              <Show when={user.institute_id}>
                <Tag level="info">
                  <span>{accountStore.institutes.find((v) => v.id === user.institute_id)?.name}</span>
                </Tag>
              </Show>
              <Popover
                size="sm"
                ghost
                square
                level="error"
                btnContent={<span class="icon-[fluent--delete-20-regular] w-5 h-5" />}
              >
                <Card contentClass="p-2 flex flex-row space-x-2 items-center">
                  <span class="icon-[fluent--warning-20-regular] w-5 h-5 text-error" />
                  <span>{t("game.admin.administrators.warningDelete")}</span>
                  <Button
                    level="error"
                    size="sm"
                    title={t("form.confirm")}
                    onClick={() => handleDeleteAdmin(user)}
                    loading={loading()}
                  >
                    <span>{t("form.confirm")}</span>
                  </Button>
                </Card>
              </Popover>
            </div>
          )}
        </For>
      </div>
    </>
  );
}

export default function () {
  return (
    <div class="flex flex-col p-3 lg:p-6 w-full items-center">
      <div class="flex flex-col w-full max-w-5xl relative space-y-2">
        <InstituteManagement />
        <div class="h-12" />
        <AdministratorsManagement />
      </div>
    </div>
  );
}
