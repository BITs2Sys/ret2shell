import { t } from "@storage/theme";
import type { Stdio } from "../stdio";
import type { Command } from "./interface";

export class Clear implements Command {
  name = "clear";
  man = t("shell.clear.man");
  func = async (io: Stdio) => {
    io.clear();
    return 0;
  };
}
