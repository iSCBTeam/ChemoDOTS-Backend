pub mod functions {
	use diesel::expression::functions::sql_function;
	use super::sql_types::*;

	sql_function! {
		#[aggregate]
		fn string_agg<ST: StringAggregatable>(value: ST, delimiter: <ST as StringAggregatable>::StringAggDelimiter) -> <ST as StringAggregatable>::StringAggResult;
	}
}

pub mod helper_types {
	use diesel::helper_types::SqlTypeOf;
	use super::functions::*;

	pub type StringAgg<Value, Delim> = string_agg::HelperType<SqlTypeOf<Value>, Value, Delim>;
}

pub mod sql_types {
	use diesel::sql_types::{Bytea, Nullable, SingleValue, SqlType, Text, is_nullable};

	pub trait StringAggregatable: SingleValue {
		type StringAggDelimiter: SqlType + SingleValue;
		type StringAggResult: SqlType + SingleValue;
	}

	impl<Value> StringAggregatable for Nullable<Value>
	where Value: StringAggregatable + SqlType<IsNull = is_nullable::NotNull>
	{
		type StringAggDelimiter = Nullable<Value>;
		type StringAggResult = Nullable<Value>;
	}

	impl StringAggregatable for Bytea {
		type StringAggDelimiter = Nullable<Bytea>;
		type StringAggResult = Bytea;
	}
	impl StringAggregatable for Text {
		type StringAggDelimiter = Nullable<Text>;
		type StringAggResult = Text;
	}
}

pub mod dsl {
	pub use super::functions::*;
	pub use super::helper_types::*;
}
