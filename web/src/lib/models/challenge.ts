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
