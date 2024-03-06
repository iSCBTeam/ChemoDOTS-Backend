use std::ops::Bound;

use diesel::backend::Backend;
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::{infix_operator, Expression};
use diesel::pg::Pg;
use diesel::query_builder::QueryId;
use diesel::serialize::ToSql;
use diesel::sql_types::{Float, Range, SqlType};

#[derive(Clone, Copy, Debug, Default, PartialEq, QueryId, SqlType)]
#[diesel(postgres_type(name = "realrange"))]
pub struct Realrange;

#[derive(Clone, Debug, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Realrange)]
pub struct RealrangeType(pub (Bound<f32>, Bound<f32>));

impl ToSql<Realrange, Pg> for RealrangeType
{
	fn to_sql<'b>(
		&'b self,
		out: &mut diesel::serialize::Output<'b, '_, Pg>,
	) -> diesel::serialize::Result {
		<(Bound<f32>, Bound<f32>) as ToSql<Range<Float>, Pg>>::to_sql(&self.0, out)
	}
}

impl FromSql<Realrange, Pg> for RealrangeType
{
	fn from_sql(
		bytes: <Pg as Backend>::RawValue<'_>,
	) -> diesel::deserialize::Result<Self> {
		<(Bound<f32>, Bound<f32>) as FromSql<Range<Float>, Pg>>::from_sql(bytes)
			.map(|range| Self(range))
	}
}

impl RealrangeType {
	pub fn new(min: Bound<f32>, max: Bound<f32>) -> Self {
		Self {
			0: (min, max),
		}
	}
}

pub mod sql_types {
	pub use super::Realrange;
}

infix_operator!(RealrangeContains, " @> ", backend: Pg);

pub trait RealrangeExpressionMethods
where
	Self: Sized,
{
	fn contains<R>(self, right: R) -> RealrangeContains<Self, R::Expression> where
		R: AsExpression<Float>;
}

impl<L> RealrangeExpressionMethods for L
where
	L: Expression<SqlType = Realrange>,
{
	fn contains<R>(self, right: R) -> RealrangeContains<Self, R::Expression> where
		R: AsExpression<Float>
	{
		RealrangeContains::new(self, right.as_expression())
	}
}