import { logout } from "@api/account";
import { HostType } from "@models/game";
import { Permission } from "@models/user";
import { useNavigate } from "@solidjs/router";
import { accountStore, refreshUser, resetUser } from "@storage/account";
import { canParticipate, gameStore } from "@storage/game";
import { t } from "@storage/theme";
import { clearToasts } from "@storage/toast";
import Avatar from "@widgets/avatar";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Link from "@widgets/link";
import Popover from "@widgets/popover";
import { Match, Show, Switch, createEffect, createSignal, untrack } from "solid-js";
import UserCodeDialog from "./user-code-dialog";

export default function UserBox() {
    createEffect(() => {
        if (accountStore.token) {
            untrack(refreshUser);
        }
    });

    const navigate = useNavigate();
    const [loading, setLoading] = createSignal(false);

    function handleLogout() {
        setLoading(true);
        setTimeout(() => {
            void logout().finally(() => {
                resetUser();
                navigate("/");
                clearToasts();
                setLoading(false);
            });
        }, 1000);
    }

    return (
        <Show
            when={accountStore.token}
            fallback={
                <Link href="/account/login" title={t("account.login.title")} ghost>
                    <span class="icon-[fluent--person-20-regular] w-5 h-5" />
                    {t("account.login.title")}
                </Link>
            }
        >
            <Popover
                title={t("account.box")}
                btnContent={
                    <Avatar
                        class="w-8 h-8"
                        src={accountStore.info?.avatar || undefined}
                        fallback={accountStore.info?.account || undefined}
                    />
                }
                square
                ghost
                popContentClass="pt-2"
            >
                <div class="flex flex-col space-y-2 max-w-64 w-[calc(100vw-1rem)]">
                    <Card contentClass="p-2 flex flex-col space-y-2">
                        <Link
                            ghost
                            class="h-16 space-x-2 flex-shrink-0 py-1 flex-nowrap"
                            justify="start"
                            href={`/users/${accountStore.info?.id}`}
                        >
                            <Avatar
                                class="w-10 h-10"
                                src={accountStore.info?.avatar || undefined}
                                fallback={accountStore.info?.account || undefined}
                                loading={loading()}
                            />
                            <div class="flex flex-col justify-center items-start">
                                <h2 class="font-bold">{accountStore.info?.nickname}</h2>
                                <span class="text-start text-base font-normal opacity-60">
                                    0x{accountStore.info?.id.toString(16).padStart(6, "0")}
                                </span>
                            </div>
                        </Link>
                    </Card>
                    <Card contentClass="p-2 flex flex-col space-y-2">
                        <UserCodeDialog />
                    </Card>
                    <Card contentClass="p-2 flex flex-col space-y-2">
                        <div class="flex flex-row space-x-2">
                            <Link href="/account/settings" ghost size="sm" justify="start" class="flex-1">
                                <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
                                <span>{t("account.settings.title")}</span>
                            </Link>
                            <Button
                                ghost
                                size="sm"
                                square
                                title={t("account.logout")}
                                onClick={handleLogout}
                                loading={loading()}
                            >
                                <Show when={!loading()}>
                                    <span class="icon-[fluent--sign-out-20-regular] w-5 h-5 text-error" />
                                </Show>
                            </Button>
                        </div>
                    </Card>
                    <Show when={gameStore.current && gameStore.current.host_type === HostType.CTFGame}>
                        <Card contentClass="p-2 flex flex-row space-x-2">
                            <Switch>
                                <Match
                                    when={
                                        accountStore.permissions.includes(Permission.Host) ||
                                        (accountStore.permissions.includes(Permission.Game) &&
                                            gameStore.current?.admins.includes(accountStore.id!))
                                    }
                                >
                                    <Button
                                        size="sm"
                                        justify="start"
                                        class="flex-1 overflow-hidden"
                                        title={t("game.adminCanNotTakePartIn")}
                                        disabled
                                    >
                                        <span class="icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
                                        <span class="flex-1 truncate text-start">
                                            {t("game.adminCanNotTakePartIn")}
                                        </span>
                                    </Button>
                                </Match>
                                <Match when={gameStore.team}>
                                    <Link
                                        href={`/games/${gameStore.current?.id}/teams/${gameStore.team?.id}`}
                                        ghost
                                        size="sm"
                                        justify="start"
                                        class="flex-1"
                                    >
                                        <span class="icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
                                        <span class="flex-1 truncate text-start">{gameStore.team?.name}</span>
                                    </Link>
                                </Match>
                                <Match when={gameStore.team}>
                                    <div />
                                </Match>
                                <Match when={canParticipate() && gameStore.team === null}>
                                    <Link
                                        href={`/games/${gameStore.current?.id}/teams/create`}
                                        ghost
                                        size="sm"
                                        justify="start"
                                        class="flex-1"
                                    >
                                        <span class="icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
                                        <span class="flex-1 truncate text-start">{t("game.team.joinGame")}</span>
                                    </Link>
                                </Match>
                                <Match when={!canParticipate()}>
                                    <Link href="#" ghost size="sm" justify="start" class="flex-1" disabled>
                                        <span class="icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
                                        <span class="flex-1 truncate text-start">{t("game.team.canNotJoin")}</span>
                                    </Link>
                                </Match>
                            </Switch>
                        </Card>
                    </Show>
                </div>
            </Popover>
        </Show>
    );
}
