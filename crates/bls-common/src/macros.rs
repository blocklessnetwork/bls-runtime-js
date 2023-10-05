
#[macro_export]
macro_rules! impl_display {
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

#[macro_export]
macro_rules! impl_query_string_conversions {
    ($prefix:expr, $struct_name:ident) => {
        impl ToString for $struct_name {
            fn to_string(&self) -> String {
                let mut query_str = serde_qs::to_string(self).expect("valid query string");
                query_str.insert_str(0, $prefix);
                query_str
            }
        }
        impl FromStr for $struct_name {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.starts_with($prefix) {
                    let stripped_s = &s[$prefix.len()..];
                    serde_qs::from_str(stripped_s).map_err(|_| "invalid query string")
                } else {
                    Err("prefix mismatch")
                }
            }
        }
    };
}
