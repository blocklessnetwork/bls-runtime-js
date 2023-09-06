
#[macro_export]
macro_rules! impl_display_via_json {
  ($t:ty) => {
      impl std::fmt::Display for $t {
          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
              let json_string = serde_json::to_string_pretty(self)
                  .map_err(|_| std::fmt::Error)?;
              write!(f, "{}", json_string)
          }
      }
  };
}
