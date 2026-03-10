import type { DateTime } from "luxon";

export type RemoteSyncState = "mirror_locked" | "detached";

export type GameSyncStatus = {
  sync_key: string | null;
  sync_token: string | null;
  readonly: boolean;
  remote_state: RemoteSyncState | null;
  remote_release_id: string | null;
  remote_first_party_base_url: string | null;
};

export type SyncRegistrySource = {
  id: number;
  name: string;
  git_url: string;
  branch: string;
  enabled: boolean;
  priority: number;
  publish_enabled: boolean;
  private_source: boolean;
  last_fetched_at: DateTime | null;
  last_error: string | null;
  created_at: DateTime;
  updated_at: DateTime;
};

export type GameReleaseSummary = {
  id: number;
  game_id: number;
  game_key: string;
  release_id: string;
  snapshot_commit: string;
  manifest_sha256: string;
  origin_role: "first_party" | "mirror";
  first_party_instance_id: string;
  first_party_base_url: string;
  published_at: DateTime;
  created_at: DateTime;
};

export type RemoteSyncInfo = {
  instance_id: string;
  base_url: string;
  protocol_version: number;
};

export type RemoteSyncGameSummary = {
  game_key: string;
  release_count: number;
};

export type RemoteSyncReleaseSummary = {
  game_key: string;
  release_id: string;
  snapshot_commit: string;
  first_party_instance_id: string;
  first_party_base_url: string;
  published_at: number;
};

export type RemoteSyncReleaseDetail = {
  game_key: string;
  release_id: string;
  snapshot_commit: string;
  manifest_sha256: string;
  manifest_body: string;
  first_party_instance_id: string;
  first_party_base_url: string;
  published_at: number;
};

export type DirectDiscoverResponse = {
  info: RemoteSyncInfo;
  games: RemoteSyncGameSummary[] | null;
  releases: RemoteSyncReleaseSummary[] | null;
  release: RemoteSyncReleaseDetail | null;
};
