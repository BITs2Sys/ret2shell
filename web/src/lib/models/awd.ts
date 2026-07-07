// BITs2CTF fork: AWD + AWDP challenge-kind types, mirroring the Rust `AwdConfig`
// and `AwdpConfig` in crates/config/src/cluster.rs. Kept in a fork-owned model file
// so upstream model changes never conflict.
import type { DateTime } from "luxon";
import type { ChallengeImage } from "./challenge";

// ---------------------------------------------------------------------------
// AWD (Attack-and-Defense): each team gets its own machine; every round the flag
// rotates and an SLA check runs; teams attack each other and submit stolen flags.
// ---------------------------------------------------------------------------
export type AwdConfig = {
  enabled: boolean;
  round_secs: number;
  internet: boolean;
  restricted: boolean | null;
  privileged: boolean | null;
  image: ChallengeImage;
  pull_secret: string | null;
  flag_path: string;
  check_command: string[] | null;
  attack_reward: number;
  defense_reward: number;
  sla_reward: number;
  timeout_secs: number;
};

export type AwdState = {
  challenge_id: number;
  last_round: number;
  last_checked_at: DateTime | null;
  last_error: string | null;
};

export type AwdInstance = {
  id: number;
  created_at: DateTime;
  challenge_id: number;
  team_id: number;
  pod_name: string;
  address: string | null;
  status: string;
};

export type AwdStatus = {
  config: AwdConfig | null;
  state: AwdState | null;
  instance: AwdInstance | null;
  round: number;
};

export type AwdSteal = {
  id: number;
  created_at: DateTime;
  challenge_id: number;
  round: number;
  attacker_team_id: number;
  attacker_name: string | null;
  victim_team_id: number;
  score: number;
  extra_id: number;
};

// ---------------------------------------------------------------------------
// AWDP (Attack-and-Defense Plus): Jeopardy-style solve/fix, no per-team machines.
// A team that solves/fixes in a round earns a persistent per-round bonus for every
// subsequent round until the game ends.
// ---------------------------------------------------------------------------
export type AwdpMode = "solve" | "fix";

export type AwdpConfig = {
  enabled: boolean;
  mode: AwdpMode;
  round_secs: number;
  total_rounds: number;
};

export type AwdpState = {
  challenge_id: number;
  last_round: number;
  last_checked_at: DateTime | null;
  last_error: string | null;
};

export type AwdpStatus = {
  config: AwdpConfig | null;
  state: AwdpState | null;
  // whether the requesting team has already secured the per-round bonus.
  solved: boolean;
  // the round in which the team first solved (drives the persistent bonus).
  solved_round: number | null;
  round: number;
};

// one per-round bonus award row (the scoreboard is the full award ledger).
export type AwdpAward = {
  id: number;
  created_at: DateTime;
  challenge_id: number;
  team_id: number;
  team_name: string | null;
  round: number;
  score: number;
  extra_id: number;
};
