import type { DateTime } from "luxon";

export type EnvConfig = {
  port: number;
  images: { tag: string; cpu: number; mem: string }[];
};

export enum InstanceState {
  Pending = 0,
  Running = 1,
  Succeeded = 2,
  Failed = 3,
}

export type Instance = {
  state: InstanceState;
  name: string;
  wsrx: string | null;
  renew_count: number;
  created_at: DateTime;
  user_id: number;
  user_name?: string;
  team_id: number | null;
  team_name?: string | null;
  challenge_id: number;
  challenge_name?: string;
  game_id: number;
  game_name?: string;
};

export type Traffic = {
  wsrx: string;
  local_addr: string;
  delay: number;
};
