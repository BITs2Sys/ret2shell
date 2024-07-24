import { type ChallengeEnv, getChallengeEnv } from "@api/game";
import { deunicode } from "@api/rpc";
import { wsrx } from "@lib/wsrx";
import type { Challenge } from "@models/challenge";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import ansiColors from "ansi-colors";
import { HTTPError } from "ky";
import type { ParseEntry } from "shell-quote";
import { link } from "../escapes";
import type { Stdio } from "../stdio";
import type { Command } from "./interface";

export class Service implements Command {
  name = "service";
  man = t("shell.service.man")!;
  func = async (io: Stdio, challenge: Challenge, args: ParseEntry[], _origin: string) => {
    const action = {
      start: this.start,
      stop: this.stop,
      restart: this.restart,
      status: this.status,
      delay: this.delay,
    };
    if (args.length !== 1 || !Object.keys(action).includes(args[0].toString().trim())) {
      io.error(t("shell.service.needAction")!);
      io.info(t("shell.service.usage")!);
      io.info(`\t${ansiColors.green(link("start", "rnix://command/service start"))}\t${t("shell.service.startTips")}`);
      io.info(`\t${ansiColors.green(link("stop", "rnix://command/service stop"))}\t${t("shell.service.stopTips")}`);
      io.info(
        `\t${ansiColors.green(link("restart", "rnix://command/service restart"))}\t${t("shell.service.restartTips")}`
      );
      io.info(
        `\t${ansiColors.green(link("status", "rnix://command/service status"))}\t${t("shell.service.statusTips")}`
      );
      io.info(`\t${ansiColors.green(link("delay", "rnix://command/service delay"))}\t${t("shell.service.delayTips")}`);
      return 1;
    }
    const env = await this.getEnv(io, challenge);
    return action[args[0].toString().trim() as "start" | "stop" | "restart" | "status" | "delay"](io, challenge, env);
  };

  async getEnv(io: Stdio, challenge: Challenge) {
    try {
      const env = await getChallengeEnv(challenge.game_id, challenge.id);
      return env;
    } catch (e) {
      if (e instanceof HTTPError) {
        const text = await e.response.text();
        io.error(`${t("shell.service.error")!}: ${text}`);
      } else {
        io.error(`${t("shell.service.error")!}: ${e}`);
      }
      return null;
    }
  }

  async start(io: Stdio, challenge: Challenge, env: ChallengeEnv | null) {
    const inst = wsrx.instances().find((instance) => instance.user_id === accountStore.id);
    if (inst) {
      if (inst.challenge_id === challenge.id) {
        this.status(io, challenge, env);
      } else {
        io.warning(t("shell.service.onlyOneInstancePersist")!);
        io.info(
          `${t("shell.service.onlyOneInstancePersistTips")}: ${link(inst.challenge_name!, `rnix://challenge/${inst.challenge_id}`)}`
        );
        io.print(
          `${t("shell.service.onlyOneInstancePersistChoice")!} [${ansiColors.green(link("yes", "rnix://command/yes"))}/${ansiColors.red(link("NO", "rnix://command/no"))}]: `
        );
        const choice = await io.input();
        io.println("");
        if (choice === "yes") {
          // TODO: Implement this
        } else {
          io.warning(t("shell.service.noActionPerformed")!);
          return 1;
        }
      }
    }
    return 0;
  }

  async stop(io: Stdio, challenge: Challenge, env: ChallengeEnv | null) {
    return 0;
  }

  async restart(io: Stdio, challenge: Challenge, env: ChallengeEnv | null) {
    return 0;
  }

  async status(io: Stdio, challenge: Challenge, env: ChallengeEnv | null) {
    if (!env) {
      io.error(t("shell.service.noEnv")!);
      return 1;
    }
    const inst = wsrx.instances().find((instance) => instance.challenge_id === challenge.id);
    const d_service_name = await deunicode(challenge.name);
    io.println(`${inst ? ansiColors.greenBright("●") : ansiColors.dim("○")} ${d_service_name}.service`);
    io.println(
      `     Loaded: loaded (~/${challenge.name}/checkers/${d_service_name}.service; ${ansiColors.yellow("disabled")}; preset: ${ansiColors.green("enabled")})`
    );
    io.println(
      `     Active: ${inst ? `${ansiColors.greenBright("active (running)")} since ${inst.created_at.toFormat("yyyy-MM-dd HH:mm:ss")}` : ansiColors.red("inactive (dead)")}`
    );
    if (inst) {
      for (const image of env.images) {
        io.println(
          `       ${ansiColors.greenBright("└─")} ${image.name}.service: ${ansiColors.greenBright("active (running)")} - ${image.description}`
        );
        io.println(
          `          Connection: ${ansiColors.blue(link(`wsrx://open?url=${inst.wsrx}&port=${image.port}`, `wsrx://open?url=${inst.wsrx}&port=${image.port}`))}`
        );
      }
    }
    io.println("");
    return 0;
  }

  async delay(io: Stdio, challenge: Challenge, env: ChallengeEnv | null) {
    return 0;
  }
}
