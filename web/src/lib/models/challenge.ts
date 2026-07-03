import type { DateTime } from "luxon";

export type Challenge = {
  id: number;
  name: string;
  updated_at: DateTime;
  content: string | null;
  hidden: boolean;
  game_id: number;
  tag: { name: string; primary: boolean }[];
  score_rule: { initial: number; minimum: number; decay: number };
  score: number;
  bucket: string | null;
  release_at: DateTime | null;
  archive_at: DateTime | null;
};

export type ChallengeImage = {
  name: string;
  tag: string;
  cpu: number;
  cpu_req: number;
  mem: string;
  mem_req: string;
  storage: string | null;
  storage_req: string | null;
  port: number | null;
  protocol?: "tcp" | "stcp" | "udp" | null;
  app_protocol?: "raw" | "http" | null;
  service_type?: "http" | "tcp" | "udp" | null;
  description: string | null;
  restricted: boolean | null;
};

export type ChallengeEnv = {
  internet: boolean;
  restricted: boolean | null;
  privileged: boolean | null;
  images: ChallengeImage[];
  pull_secret: string | null;
};

export type FixConfig = {
  enabled: boolean;
  max_attempts: number;
  fix_script: string;
  upload_path: string;
  target_container: string | null;
  target_port: number | null;
  tester: ChallengeImage | null;
  tester_command: string[] | null;
  result_env: string;
  success_value: string;
  timeout_secs: number;
  pull_secret: string | null;
};

export type FixStatus = {
  config: FixConfig | null;
  attempts_used: number;
  attempts_remaining: number | null;
  solved: boolean;
};

export type KohMode = "agent_http" | "round_rank_http" | "game_elo";

export type KohEloConfig = {
  calibration_rounds: number;
  initial_rating: number;
  k_factor: number;
};

export type KohConfig = {
  enabled: boolean;
  mode: KohMode;
  interval_secs: number;
  round_secs: number;
  total_rounds: number;
  reward: number;
  rank_count: number;
  rank_percentages: number[];
  status_url: string | null;
  status_path: string;
  api_key: string | null;
  agent_port: number | null;
  target_port: number | null;
  timeout_secs: number;
  auto_start: boolean;
  elo: KohEloConfig | null;
};

export type KohIdentifier = {
  id: number;
  created_at: DateTime;
  challenge_id: number;
  team_id: number;
  identifier: string;
};

export type KohState = {
  challenge_id: number;
  current_identifier: string | null;
  current_team_id: number | null;
  last_checked_at: DateTime | null;
  last_awarded_at: DateTime | null;
  last_error: string | null;
};

export type KohTarget = {
  state: string;
  name: string;
  traffic: string;
  ports: number[];
  target_port: number | null;
  exposed_ports: { name: string; address: string }[] | null;
};

export type KohStatus = {
  config: KohConfig | null;
  state: KohState | null;
  identifier: KohIdentifier | null;
  target: KohTarget | null;
};

export type KohEvent = {
  id: number;
  created_at: DateTime;
  challenge_id: number;
  challenge_name: string | null;
  team_id: number | null;
  team_name: string | null;
  previous_team_id: number | null;
  identifier: string | null;
  status: string;
  message: string | null;
  score_delta: number;
  tick: number | null;
};

export type KohScore = {
  team_id: number;
  team_name: string | null;
  score: number;
  last_awarded_at: DateTime | null;
};
