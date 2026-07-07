// BITs2CTF fork: ISW (Internal Security Warfare) range-mode config, mirroring the
// Rust `IswConfig` in crates/config/src/cluster.rs. Kept in a fork-owned model
// file so upstream model changes never conflict.
export type IswConfig = {
  enabled: boolean;
  range_template: string;
  vm: string;
  guest_path: string;
  owner: string | null;
  mode: string;
  rotate: boolean;
};
