import { Permission } from "@models/user";
import xdsecMascotNormal from "@assets/imgs/xdsec-mascot-normal.webp";
import xdsecMascotUnsee from "@assets/imgs/xdsec-mascot-unsee.webp";
import { A } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { gameStore } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Input from "@widgets/input";
import { Show } from "solid-js";

export default function () {
    return (
        <div class="flex flex-col min-h-full relative">
            <div class="flex flex-col flex-1 p-3 lg:p-6 space-y-4">
                <div class="self-start flex-row max-w-[calc(100%-4rem)] flex items-center">
                    <img src={xdsecMascotNormal} width={40} height={40} alt="ΦωΦ" />
                    <div class="w-4" />
                    <Card contentClass="p-2">
                        <p class="text-wrap">{t("game.challenge.hammerTips")}</p>
                    </Card>
                </div>
                <div class="self-start flex-row max-w-[calc(100%-4rem)] flex items-center">
                    <img src={xdsecMascotUnsee} width={40} height={40} alt=">ω<" />
                    <div class="w-4" />
                    <Card contentClass="p-2">
                        <p class="text-wrap">{t("game.challenge.hammerTips2")}</p>
                    </Card>
                </div>
            </div>
            <div class="sticky bottom-0 p-3 lg:p-6">
                <Input
                    placeholder={t("game.challenge.hammerInput")}
                    extraBtn={
                        <Button class="!rounded-l-none">
                            <span class="icon-[fluent--send-20-regular] w-5 h-5" />
                        </Button>
                    }
                />
            </div>
            <Show
                when={
                    accountStore.id &&
                    accountStore.permissions.includes(Permission.Game) &&
                    gameStore.current?.admins.includes(accountStore.id)
                }
            >
                <div class="absolute top-0 left-0 w-full h-full backdrop-blur flex items-center justify-center">
                    <A
                        class="font-bold hover:underline hover:text-primary flex items-center space-x-2"
                        href={`/games/${gameStore.current?.id}/admin/hammers`}
                    >
                        <span class="icon-[fluent--open-20-regular] w-5 h-5" />
                        <span>{t("game.admin.hammer.shouldGoto")}</span>
                    </A>
                </div>
            </Show>
        </div>
    );
}
