// BITs2CTF fork: ISW range-mode admin models (mirror the isw_* SeaORM entities).
import type { DateTime } from "luxon";

export type IswHost = {
  id: number;
  created_at: DateTime;
  name: string;
  address: string;
  api_port: number;
  os: string;
  fingerprint: string | null;
  enabled: boolean;
  status: string;
  free_mem_mb: number | null;
  last_heartbeat: DateTime | null;
};

export type IswVmSpec = {
  logical_name: string;
  guest_os: string;
  vmx: string;
  creds_ref: string;
};

export type IswTopology = {
  vmnet: string;
  vms: IswVmSpec[];
  vpn: { kind: string; server_endpoint: string; subnet: string } | null;
};

export type IswRangeTemplate = {
  id: number;
  created_at: DateTime;
  game_id: number;
  name: string;
  brief: string;
  topology: IswTopology;
};

export type IswRange = {
  id: number;
  created_at: DateTime;
  template_id: number;
  host_id: number;
  group_index: number;
  name: string;
  status: string;
  armed_at: DateTime | null;
  snapshot_name: string | null;
  last_error: string | null;
};

export type IswVm = {
  id: number;
  range_id: number;
  logical_name: string;
  guest_os: string;
  vmx_path: string;
  ip: string | null;
  power_state: string;
  tools_state: string;
};

export type IswFlag = {
  id: number;
  range_id: number;
  challenge_id: number;
  vm_id: number | null;
  guest_path: string;
  value_hash: string;
  round: number;
  verified: boolean;
  last_error: string | null;
};

export type RangeDetail = {
  range: IswRange;
  vms: IswVm[];
  flags: IswFlag[];
};

export type FlagArmResult = {
  challenge_id: number;
  vm: string;
  guest_path: string;
  ok: boolean;
  message: string | null;
};

export type ArmReport = {
  range_id: number;
  flags: FlagArmResult[];
};

export type RangeHealth = {
  free_mem_mb: number;
  vmrun_ok: boolean;
  vms: { logical_name: string; power_state: string; tools_state: string; ip: string | null }[];
};
