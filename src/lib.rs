mod dynamic_filters;
mod inner_statement;
mod stack_overflow;
// Filters for "numbers"
enum NumberFilter<T> {
    Equal(T),
    NotEqual(T),
    GreaterThen(T),
    LowerThen(T),
    IsNull,
    IsNotNull,
}

macro_rules! number_filter {
    ($filter:ident, $dsl_field:expr ) => {{
        match $filter {
            NumberFilter::Equal(value) => Box::new($dsl_field.eq(value).nullable()),
            NumberFilter::NotEqual(value) => Box::new($dsl_field.ne(value).nullable()),
            NumberFilter::GreaterThen(value) => Box::new($dsl_field.gt(value).nullable()),
            NumberFilter::LowerThen(value) => Box::new($dsl_field.lt(value).nullable()),
            NumberFilter::IsNull => Box::new($dsl_field.is_null().nullable()),
            NumberFilter::IsNotNull => Box::new($dsl_field.is_not_null().nullable()),
        }
    }};
}

enum StringFilter {
    Equal(String),
    NotEqual(String),
    Like(String),
    In(Vec<String>),
}

macro_rules! string_filter {
    ($filter:ident, $dsl_field:expr ) => {{
        match $filter {
            StringFilter::Equal(value) => Box::new($dsl_field.eq(value).nullable()),
            StringFilter::NotEqual(value) => Box::new($dsl_field.ne(value).nullable()),
            StringFilter::Like(value) => Box::new($dsl_field.like(value).nullable()),
            StringFilter::In(value) => Box::new($dsl_field.eq_any(value).nullable()),
        }
    }};
}

enum BooleanFilter {
    True,
    False,
    IsNull,
    IsNotNull,
}

macro_rules! boolean_filter {
    ($filter:ident, $dsl_field:expr ) => {{
        match $filter {
            BooleanFilter::True => Box::new($dsl_field.eq(true).nullable()),
            BooleanFilter::False => Box::new($dsl_field.eq(false).nullable()),
            BooleanFilter::IsNull => Box::new($dsl_field.is_null().nullable()),
            BooleanFilter::IsNotNull => Box::new($dsl_field.is_not_null().nullable()),
        }
    }};
}

enum AndOr {
    And,
    Or,
}

use boolean_filter;
use number_filter;
use string_filter;
