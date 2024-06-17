import { generateAccountCode, getAccountCode } from "@api/account";
import { accountStore, setAccountStore } from "@storage/account";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Dialog from "@widgets/dialog";
import Timer from "@widgets/timer";
import type { HTTPError } from "ky";
import type { DateTime } from "luxon";
import { Match, Switch, createEffect, createSignal, untrack } from "solid-js";

export default function UserCodeDialog() {
    const [code, setCode] = createSignal(null as { code: number; generate_at: DateTime } | null);
    const [loadingCode, setLoadingCode] = createSignal(true);
    function getCode() {
        setLoadingCode(true);
        void getAccountCode()
            .then(setCode)
            .catch(() => setCode(null))
            .finally(() => setLoadingCode(false));
    }
    function refreshCode() {
        if (!accountStore.warnedCodeGeneration) {
            setAccountStore({ warnedCodeGeneration: true });
        }
        setCode(null);
        setLoadingCode(true);
        setTimeout(() => {
            generateAccountCode()
                .then(setCode)
                .catch((err: HTTPError) => {
                    err.response.text().then((text) => {
                        addToast({
                            level: "error",
                            description: text,
                            duration: 5000,
                        });
                    });
                })
                .finally(() => setLoadingCode(false));
        }, 500);
    }
    createEffect(() => {
        if (accountStore.token) {
            untrack(getCode);
        } else {
            setCode(null);
        }
    });
    return (
        <Dialog
            size="sm"
            justify="start"
            ghost
            btnContent={
                <>
                    <span class="icon-[fluent--person-link-20-regular] w-5 h-5" />
                    <span>{t("account.code.title")}</span>
                </>
            }
        >
            <div class="flex flex-col w-64 space-y-2">
                <div class="w-full min-h-36 flex flex-col items-center justify-center space-y-2">
                    <Switch>
                        <Match when={!accountStore.warnedCodeGeneration}>
                            <span class="icon-[fluent--warning-20-regular] w-10 h-10 flex-shrink-0 text-warning" />
                            <span class="text-warning">{t("account.code.warn")}</span>
                        </Match>
                        <Match when={code()}>
                            <span class="font-extrabold text-5xl tracking-widest">
                                {code()?.code.toString(16).toUpperCase().padStart(6, "0")}
                            </span>
                            <Timer class="opacity-80 font-bold" end={code()!.generate_at.plus({ seconds: 300 })} />
                        </Match>
                        <Match when={true}>
                            <span class="icon-[fluent--person-link-20-regular] w-10 h-10 opacity-60" />
                            <span class="opacity-60">{t("account.code.null")}</span>
                        </Match>
                    </Switch>
                </div>
                <Button
                    level="primary"
                    square
                    title={t("account.code.refresh")}
                    onClick={refreshCode}
                    loading={loadingCode()}
                    disabled={loadingCode()}
                >
                    <Switch>
                        <Match when={!accountStore.warnedCodeGeneration}>
                            <span class="icon-[fluent--emoji-hand-20-regular] w-5 h-5" />
                            <span class="truncate">{t("account.code.confirmWarn")}</span>
                        </Match>
                        <Match when={loadingCode()}>
                            <span class="truncate">{t("form.loading")}</span>
                        </Match>
                        <Match when={true}>
                            <span class="icon-[fluent--arrow-clockwise-20-regular] w-5 h-5" />
                            <span class="truncate">{t("account.code.refresh")}</span>
                        </Match>
                    </Switch>
                </Button>
            </div>
        </Dialog>
    );
}
