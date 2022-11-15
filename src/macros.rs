macro_rules! unwrap {
	($expr:expr, $fail_body:block) => {
		match $expr {
			::std::option::Option::Some(val) => val,
			::std::option::Option::None => $fail_body,
			}
	};
}
