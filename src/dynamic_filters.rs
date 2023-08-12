/*
    Adding and/or and grouped/nested filter
*/

use diesel::{connection::SimpleConnection, prelude::*, sql_types::Bool, sqlite::Sqlite};

table! {
    test (id) {
        id -> Text,
        number_field -> Integer,
        text_field -> Text,
        bool_field -> Bool
    }
}

// Filters for "numbers"
enum NumberFilter {
    Equal(i32),
    NotEqual(i32),
    GreaterThen(i32),
    LowerThen(i32),
}
// Filters for "strings"
enum StringFilter {
    Equal(String),
    NotEqual(String),
    Like(String),
}

enum AndOr {
    And,
    Or,
}

#[allow(non_camel_case_types)]
enum Condition {
    number_field(NumberFilter),
    text_field(StringFilter),
    bool_field(bool),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

// Need this type for common condition expressions
type BoxedCondition = Box<dyn BoxableExpression<test::dsl::test, Sqlite, SqlType = Bool>>;

fn create_filter(conditions: Vec<Condition>, and_or: AndOr) -> Option<BoxedCondition> {
    conditions
        .into_iter()
        // Map into array of boxed conditions
        .filter_map::<BoxedCondition, _>(|condition| {
            Some(match condition {
                Condition::number_field(f) => match f {
                    NumberFilter::Equal(value) => Box::new(test::dsl::number_field.eq(value)),
                    NumberFilter::NotEqual(value) => Box::new(test::dsl::number_field.ne(value)),
                    NumberFilter::GreaterThen(value) => Box::new(test::dsl::number_field.gt(value)),
                    NumberFilter::LowerThen(value) => Box::new(test::dsl::number_field.lt(value)),
                },
                Condition::text_field(f) => match f {
                    StringFilter::Equal(value) => Box::new(test::dsl::text_field.eq(value)),
                    StringFilter::NotEqual(value) => Box::new(test::dsl::text_field.ne(value)),
                    StringFilter::Like(value) => Box::new(test::dsl::text_field.like(value)),
                },
                Condition::bool_field(value) => Box::new(test::dsl::bool_field.eq(value)),
                Condition::And(conditions) => match create_filter(conditions, AndOr::And) {
                    Some(boxed_condition) => boxed_condition,
                    None => return None,
                },
                Condition::Or(conditions) => match create_filter(conditions, AndOr::Or) {
                    Some(boxed_condition) => boxed_condition,
                    None => return None,
                },
            })
        })
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

fn create__and_filter(conditions: Vec<Condition>) -> Option<BoxedCondition> {
    create_filter(conditions, AndOr::And)
}

#[test]
fn test() {
    let mut connection = SqliteConnection::establish("file:test?mode=memory&cache=shared").unwrap();
    // See
    connection
        .batch_execute(
            r#"
            CREATE TABLE test (
                id TEXT PRIMARY KEY,
                number_field NUMBER NOT NULL,
                text_field TEXT NOT NULL DEFAULT '',
                bool_field BOOL NOT NULL DEFAULT false
            );

            INSERT INTO test 
              (id, number_field) 
            VALUES
              ('1', 1);
            
            INSERT INTO test 
              (id, number_field) 
            VALUES
              ('2', 2);

            INSERT INTO test 
              (id, number_field) 
            VALUES
              ('3', 3);
        "#,
        )
        .unwrap();

    let condition =
        create__and_filter(vec![Condition::number_field(NumberFilter::Equal(1))]).unwrap();
    let result = vec!["1".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition =
        create__and_filter(vec![Condition::number_field(NumberFilter::NotEqual(1))]).unwrap();
    let result = vec!["2".to_string(), "3".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .order_by(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create__and_filter(vec![
        Condition::number_field(NumberFilter::GreaterThen(1)),
        Condition::number_field(NumberFilter::LowerThen(3)),
    ])
    .unwrap();
    let result = vec!["2".to_string()];

    connection
        .batch_execute(
            r#"
            INSERT INTO test 
              (id, number_field, text_field, bool_field) 
            VALUES
              ('4.1', 4, '4.1', false);

            INSERT INTO test 
              (id, number_field, text_field, bool_field) 
            VALUES
              ('4.2', 4, '4.2', true);
        "#,
        )
        .unwrap();

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create__and_filter(vec![
        Condition::number_field(NumberFilter::GreaterThen(1)),
        Condition::text_field(StringFilter::Like("%4%".to_string())),
        Condition::bool_field(true),
    ])
    .unwrap();
    let result = vec!["4.2".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    // To test and/or and grouped/nested filter, we try this scenario
    // when bool_field = true
    // bool_field = true or (bool_field = true and bool_field = false) === true
    // vs
    // (bool_field = true or bool_field = true) and bool_field = false === false

    connection
        .batch_execute(
            r#"
            INSERT INTO test 
              (id, number_field, bool_field) 
            VALUES
              ('5', 5, true);
        "#,
        )
        .unwrap();

    let condition = create__and_filter(vec![
        Condition::number_field(NumberFilter::Equal(5)),
        Condition::Or(vec![
            Condition::bool_field(true),
            Condition::And(vec![
                Condition::bool_field(true),
                Condition::bool_field(false),
            ]),
        ]),
    ])
    .unwrap();

    let result = vec!["5".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create__and_filter(vec![
        Condition::number_field(NumberFilter::Equal(5)),
        Condition::And(vec![
            Condition::Or(vec![
                Condition::bool_field(true),
                Condition::bool_field(true),
            ]),
            Condition::bool_field(false),
        ]),
    ])
    .unwrap();

    // No return
    let result: Vec<String> = vec![];

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );
}
