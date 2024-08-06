use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckerError {
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Missing checker script: {0}")]
  MissingCheckerScript(String),
  #[error("Rune context error: {0}")]
  RuneError(#[from] rune::ContextError),
  #[error("Can not load script source: {0}")]
  SourceError(#[from] rune::source::FromPathError),
  #[error("Can not build script unit: {0}")]
  BuildError(#[from] rune::BuildError),
  #[error("Can not alloc script engine runtime: {0}")]
  AllocError(#[from] rune::alloc::Error),
  #[error("Executed script error: {0}")]
  ExecError(#[from] rune::runtime::VmError),
  #[error("Missing fields from script result: {0}")]
  MissingResultField(String),
  #[error("Script error: {0}")]
  ScriptError(String),
  #[error("Compile error: {0}")]
  CompileError(String),
  #[error("Missing function: {0}")]
  MissingFunction(String),
  #[error("Failed to emit diagnostics: {0}")]
  DiagnosticsError(#[from] rune::diagnostics::EmitError),
  #[error("String UTF-8 decode error: {0}")]
  FromUtf8Error(#[from] std::string::FromUtf8Error),
}
