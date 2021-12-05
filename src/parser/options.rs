#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParserOption {
  Debug,
}

impl ParserOption {
  #[allow(dead_code)]
  fn is_positive_answer<S: AsRef<str>>(s: S) -> bool {
    match s.as_ref().to_lowercase().as_str() {
      "yes" | "y" | "1" | "on" | "ok" => true,
      _ => false
    }
  }

  #[allow(dead_code)]
  fn is_negative_answer<S: AsRef<str>>(s: S) -> bool {
    match s.as_ref().to_lowercase().as_str() {
      "no" | "n" | "0" | "off" => true,
      _ => false
    }
  }

  #[allow(dead_code)]
  fn env_var<S: AsRef<str>>(n: S) -> Option<String> {
    let runtime = std::env::var(n.as_ref());
    if runtime.is_ok() {
      return Some(runtime.unwrap());
    }
    None
  }

  #[allow(dead_code)]
  pub fn from_env() -> Vec<ParserOption> {
    let raw_parser_debug = Self::env_var("VM_PARSER_DEBUG");
    let mut opts: Vec<ParserOption> = vec![];
    if raw_parser_debug.is_some() && !Self::is_negative_answer(raw_parser_debug.unwrap()) {
      opts.push(ParserOption::Debug);
    }
    return opts;
  }
}