import { accountStore } from "@/lib/storage/account";
import { generateRandomName } from "@/lib/utils/random-names";
import Article from "@/lib/widgets/article";
import Button from "@/lib/widgets/button";
import Card from "@/lib/widgets/card";
import Dialog from "@/lib/widgets/dialog";
import Input from "@/lib/widgets/input";
import Link from "@/lib/widgets/link";
import Popover from "@/lib/widgets/popover";
import { createForm, custom, maxLength, minLength, required, setValue } from "@modular-forms/solid";
import { A, useNavigate } from "@solidjs/router";
import { canParticipate, gameStore } from "@storage/game";
import { Title } from "@storage/header";
import { fullTheme, t, themeStore } from "@storage/theme";
import { addToast } from "@storage/toast";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { Show, createEffect, createSignal, untrack } from "solid-js";

type TeamCreateForm = {
    name: string;
    accepted: boolean;
};

export default function () {
    const navigate = useNavigate();
    const [customDisabled, setCustomDisabled] = createSignal(false);
    const [form, { Form, Field }] = createForm<TeamCreateForm>();
    createEffect(() => {
        if (gameStore.current && !canParticipate()) {
            addToast({
                level: "warning",
                description: t("game.canNotParticipate")!,
                duration: 5000,
            });
            navigate(`/games/${gameStore.current.id}`, { replace: true });
        }
    });
    createEffect(() => {
        if (gameStore.current) {
            untrack(() => {
                setCustomDisabled(gameStore.current?.team_size === 1);
                setValue(form, "name", accountStore.nickname!);
            });
        }
    });
    const [loading, setLoading] = createSignal(false);
    function onSubmit(data: TeamCreateForm) {}
    const [generator, setGenerator] = createSignal<"chuunibyou" | "hacker">("hacker");
    const [content, setContent] = createSignal(null as null | string);
    const comps = import.meta.glob("../../_blocks/contents/*.md");
    createEffect(() => {
        if (themeStore.locale) {
            untrack(() => {
                const match = comps[`../../_blocks/contents/welcome.${themeStore.locale}.md`];
                match().then((content) => {
                    setContent((content as { default: string }).default);
                });
            });
        }
    });
    return (
        <>
            <Title title={`${t("game.team.create.title")} - ${gameStore.current?.name || "CTF"}`} />
            <div class="flex-1 flex flex-col items-center md:justify-center p-3 md:p-6">
                <Card
                    class="w-full max-w-xl"
                    contentClass="p-6 flex flex-col md:flex-row space-y-2 space-x-0 md:space-x-6 md:space-y-0"
                >
                    <Form onSubmit={onSubmit} class="md:w-0 flex-1 flex-shrink-0 flex flex-col space-y-2">
                        <h2 class="font-bold text-center">{t("game.team.create.title")}</h2>
                        <Field
                            name="name"
                            validate={[
                                required(t("game.team.create.nameRequired")!),
                                maxLength(32, t("game.team.create.nameMaxLength")!),
                            ]}
                        >
                            {(field, props) => (
                                <Input
                                    icon={<span class="icon-[fluent--flag-20-regular] w-5 h-5" />}
                                    title={
                                        customDisabled()
                                            ? t("game.team.create.customDisabled")
                                            : t("game.team.create.name")
                                    }
                                    placeholder={t("game.team.create.namePlaceholder")}
                                    {...props}
                                    value={field.value}
                                    error={field.error}
                                    required
                                    disabled={customDisabled()}
                                    extraBtn={
                                        <>
                                            <Button
                                                class="!rounded-none"
                                                square
                                                disabled={customDisabled()}
                                                onClick={() => {
                                                    generateRandomName(generator()).then((name: string) => {
                                                        setValue(form, "name", name);
                                                    });
                                                }}
                                            >
                                                <span class="icon-[fluent--diversity-20-regular] w-5 h-5" />
                                            </Button>
                                            <Popover
                                                class="!rounded-l-none"
                                                square
                                                disabled={customDisabled()}
                                                btnContent={<span class="icon-[fluent--settings-20-regular] w-5 h-5" />}
                                            >
                                                <Card contentClass="p-2 flex flex-col space-y-2">
                                                    <Button
                                                        ghost
                                                        size="sm"
                                                        justify="start"
                                                        onClick={() => setGenerator("hacker")}
                                                    >
                                                        <span
                                                            class={`icon-[fluent--diversity-20-regular] w-5 h-5 ${generator() === "hacker" ? "text-primary" : ""}`}
                                                        />
                                                        <span>Hacker Names</span>
                                                    </Button>
                                                    <Button
                                                        ghost
                                                        size="sm"
                                                        justify="start"
                                                        onClick={() => setGenerator("chuunibyou")}
                                                    >
                                                        <span
                                                            class={`icon-[fluent--diversity-20-regular] w-5 h-5 ${generator() === "chuunibyou" ? "text-primary" : ""}`}
                                                        />
                                                        <span>{t("game.team.create.chuunibyouGenerator")}</span>
                                                    </Button>
                                                </Card>
                                            </Popover>
                                        </>
                                    }
                                />
                            )}
                        </Field>
                        <Field
                            name="accepted"
                            type="boolean"
                            validate={[required(t("game.team.create.acceptedRequired")!)]}
                        >
                            {(field, props) => (
                                <>
                                    <input type="checkbox" class="hidden" {...props} checked={field.value} />
                                    <Dialog
                                        justify="start"
                                        ghost
                                        level={field.error ? "error" : null}
                                        btnContent={
                                            <>
                                                <Show
                                                    when={field.value}
                                                    fallback={
                                                        <span class="icon-[fluent--checkmark-circle-20-regular] w-5 h-5" />
                                                    }
                                                >
                                                    <span class="icon-[fluent--checkmark-circle-20-filled] w-5 h-5 text-primary" />
                                                </Show>
                                                <span>{t("game.team.create.acceptRules")}</span>
                                            </>
                                        }
                                    >
                                        <div class="w-[calc(100vw-3rem)] max-w-5xl max-h-[calc(100vh-3rem)] overflow-scroll">
                                            <h2 class="text-center text-3xl font-bold p-4 pb-0">
                                                {t("game.team.create.rulesTitle")}
                                            </h2>
                                            <Article class="self-center" content={content() || ""} noExtraPaddings />
                                        </div>
                                        <div class="flex space-x-2">
                                            <Button
                                                class="flex-1"
                                                level="success"
                                                onClick={() => setValue(form, "accepted", true)}
                                                disabled={field.value}
                                            >
                                                <span>
                                                    <Show when={field.value}>{t("game.team.create.accepted")}</Show>
                                                    <Show when={!field.value}>{t("game.team.create.accept")}</Show>
                                                </span>
                                            </Button>
                                            <Show when={field.value}>
                                                <Button level="error" onClick={() => setValue(form, "accepted", false)}>
                                                    <span>{t("game.team.create.reject")}</span>
                                                </Button>
                                            </Show>
                                        </div>
                                    </Dialog>
                                </>
                            )}
                        </Field>
                        <Button type="submit" level="primary" loading={loading()} disabled={loading()}>
                            {t("form.create")}
                        </Button>
                        <A
                            href={`/games/${gameStore.current?.id}/teams/join`}
                            class="flex items-center space-x-2 justify-center h-12 hover:underline opacity-60"
                        >
                            <span>{t("game.team.joinTips")}</span>
                            <span class="icon-[fluent--arrow-right-20-regular] w-5 h-5" />
                        </A>
                    </Form>
                </Card>
            </div>
        </>
    );
}
