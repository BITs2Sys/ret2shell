import type { User } from "@models/user";
import { fullTheme } from "@storage/theme";
import Avatar from "@widgets/avatar";
import Divider from "@widgets/divider";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";

export default function (props: { user: User | null; loading?: boolean }) {
  return (
    <div class="w-full h-full overflow-hidden">
      <OverlayScrollbarsComponent
        options={{
          scrollbars: {
            theme: `os-theme-${fullTheme()}`,
            autoHide: "scroll",
          },
        }}
        class="relative w-full h-full print:h-auto print:overflow-auto"
        defer
      >
        <div class="flex flex-col space-y-2 p-3 lg:p-6">
          <div class="flex flex-row space-x-4 lg:space-x-6 p-2 lg:p-4 items-center">
            <Avatar
              class="w-12 h-12"
              src={props.user?.avatar || undefined}
              fallback={props.user?.nickname}
              loading={props.loading}
            />
            <div class="flex-1 flex flex-col space-y-1 justify-center">
              <h2 class="font-bold">{props.user?.nickname}</h2>
              <p class="opacity-60">
                <span>{props.user?.account}</span>
                <span>#</span>
                <span>{props.user?.id.toString(16).padStart(6, "0")}</span>
              </p>
            </div>
          </div>
          <Divider />
        </div>
      </OverlayScrollbarsComponent>
    </div>
  );
}
