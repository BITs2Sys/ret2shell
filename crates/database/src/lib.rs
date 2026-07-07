mod entities;

pub use entities::{
  article, audit, calendar, challenge, chat, comment, config, extra, game, hint, institute, ip,
  media, notification, oauth, oauth_provider, submission, team, user, user2_ip, user2_team,
};
// BITs2CTF fork: fork-owned entities re-exported append-only so the generated
// block above stays pristine for upstream merges.
pub use entities::isw::{
  isw_assignment, isw_flag, isw_host, isw_range, isw_range_template, isw_vm, isw_vpn_peer,
};
pub use entities::awd::{
  awd_instance, awd_round, awd_state, awd_steal, awdp_award, awdp_solve, awdp_state,
};
pub use entities::{koh_award, koh_event, koh_identifier, koh_state};
pub use sea_orm::DbErr;
