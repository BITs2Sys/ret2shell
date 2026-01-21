import { getProfile } from "@api/account";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import ansiColors from "ansi-colors";
import { link } from "../escapes";
import type { Stdio } from "../stdio";
import type { Command } from "./interface";

export class Whoami implements Command {
  name = "whoami";
  man = t("shell.whoami.man");
  func = async (io: Stdio) => {
    try {
      if (!accountStore.token) {
        throw new Error("Not logged in");
      }
      const info = await getProfile();

      const email = ansiColors.dim(link(info.email || "guest@private.ret.sh.cn", `mailto:${info.email}`));
      io.println(
        `${ansiColors.blue(accountStore.account || "guest")} (${accountStore.nickname}) ${ansiColors.dim("<")}${ansiColors.dim(
          email
        )}${ansiColors.dim(">")}`
      );
    } catch {
      io.println("[UNKNOWN] not logged in");
      return 1;
    }
    return 0;
  };
}
