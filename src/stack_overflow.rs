/*
    Macros and helpers
    Null condition
*/

use super::*;
use diesel::{connection::SimpleConnection, prelude::*, sql_types::Bool, sqlite::Sqlite};

table! {
    foo (id) {
        id -> Text,
        bar -> Integer,
    }
}

#[allow(non_camel_case_types)]
enum Condition {
    bar(i32),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

type Source = foo::dsl::foo;
// Need this type for common condition expressions
type BoxedCondition = Box<dyn BoxableExpression<Source, Sqlite, SqlType = Bool>>;

impl Condition {
    fn to_boxed_condition(self) -> Option<BoxedCondition> {
        Some(match self {
            // Here we box the condition
            Condition::bar(value) => Box::new(foo::dsl::bar.eq(value)),
            // For and/or
            Condition::And(conditions) => match create_filter(conditions, AndOr::And) {
                Some(boxed_condition) => boxed_condition,
                None => return None,
            },
            Condition::Or(conditions) => match create_filter(conditions, AndOr::Or) {
                Some(boxed_condition) => boxed_condition,
                None => return None,
            },
        })
    }
}

// Iterate over conditions and box them
fn create_filter(conditions: Vec<Condition>, and_or: AndOr) -> Option<BoxedCondition> {
    conditions
        .into_iter()
        // Map into array of boxed conditions
        .filter_map::<BoxedCondition, _>(Condition::to_boxed_condition)
        // Reduce to a boxed_condition1.and(boxed_condition2).and(boxed_condition3)...
        .fold(None, |boxed_conditions, boxed_condition| {
            Some(match boxed_conditions {
                Some(bc) => match and_or {
                    AndOr::And => Box::new(bc.and(boxed_condition)),
                    AndOr::Or => Box::new(bc.or(boxed_condition)),
                },
                None => boxed_condition,
            })
        })
}

#[test]
fn test() {
    let mut connection = SqliteConnection::establish("file:test?mode=memory&cache=shared").unwrap();

    connection
        .batch_execute(
            r#"
            CREATE TABLE foo (
                id TEXT PRIMARY KEY,
                bar NUMBER NOT NULL
            );

            INSERT INTO foo 
              (id, bar) 
            VALUES
              ('1', 1);
        "#,
        )
        .unwrap();

    // bar = 1 or (bar = 1 and bar = 0) === true
    // vs
    // (bar = 1 or bar = 1) and bar = 0 === false

    let condition = create_filter(
        vec![Condition::Or(vec![
            Condition::bar(1),
            Condition::And(vec![Condition::bar(1), Condition::bar(0)]),
        ])],
        AndOr::And,
    )
    .unwrap();

    let result = vec!["1".to_string()];

    assert_eq!(
        result,
        foo::dsl::foo
            .filter(condition)
            .select(foo::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create_filter(
        vec![Condition::And(vec![
            Condition::Or(vec![Condition::bar(1), Condition::bar(1)]),
            Condition::bar(0),
        ])],
        AndOr::And,
    )
    .unwrap();

    // No return
    let result: Vec<String> = vec![];

    assert_eq!(
        result,
        foo::dsl::foo
            .filter(condition)
            .select(foo::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );
}
