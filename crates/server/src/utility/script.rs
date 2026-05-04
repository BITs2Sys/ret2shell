use r2s_engine::{DiagnosticKind, DiagnosticMarker};

pub fn has_diagnostic_error(lint: &[DiagnosticMarker]) -> bool {
  diagnostic_error_count(lint) > 0
}

pub fn diagnostic_error_count(lint: &[DiagnosticMarker]) -> usize {
  lint
    .iter()
    .filter(|marker| matches!(&marker.kind, DiagnosticKind::Error))
    .count()
}
