import { t } from "@storage/theme";
import type { ParseEntry } from "shell-quote";
import type { Stdio } from "../stdio";
import type { Command } from "./interface";

export class Echo implements Command {
  name = "echo";
  man = t("shell.echo.man");
  func = async (io: Stdio, _args: ParseEntry[], origin: string) => {
    io.println(origin.replace("echo", "").trim());
    return 0;
  };
}
