import type { Challenge } from "@models/challenge";
import type { Game } from "@models/game";
import type { Team } from "@models/team";
import type { ParseEntry } from "shell-quote";
import type { Stdio } from "../stdio";

export interface Command {
  name: string;
  func(
    io: Stdio,
    args: ParseEntry[],
    origin: string,
    env: {
      game?: Game;
      team?: Team;
      challenge?: Challenge;
    }
  ): Promise<number>;
  man: string;
}
