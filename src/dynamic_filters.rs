/*
    Macros and helpers
    Null condition
*/

use super::*;
use diesel::{
    connection::SimpleConnection,
    helper_types::LeftJoinQuerySource,
    prelude::*,
    sql_types::{Bool, Nullable},
    sqlite::Sqlite,
};

table! {
    test (id) {
        id -> Text,
        number_field -> Integer,
        text_field -> Text,
        bool_field -> Bool
    }
}

table! {
    join_to_test (id) {
        id -> Text,
        test_id -> Text,
        double_field -> Double,
    }
}

joinable!(join_to_test -> test (test_id));
allow_tables_to_appear_in_same_query!(test, join_to_test);

#[allow(non_camel_case_types)]
enum Condition {
    number_field(NumberFilter<i32>),
    double_field(NumberFilter<f64>),
    text_field(StringFilter),
    bool_field(BooleanFilter),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

type Source = LeftJoinQuerySource<test::dsl::test, join_to_test::dsl::join_to_test>;
// Need this type for common condition expressions
type BoxedCondition = Box<dyn BoxableExpression<Source, Sqlite, SqlType = Nullable<Bool>>>;

impl Condition {
    fn to_boxed_condition(self) -> Option<BoxedCondition> {
        Some(match self {
            Condition::number_field(f) => number_filter!(f, test::dsl::number_field),
            Condition::double_field(f) => number_filter!(f, join_to_test::dsl::double_field),
            Condition::text_field(f) => string_filter!(f, test::dsl::text_field),
            Condition::bool_field(value) => boolean_filter!(value, test::dsl::bool_field),
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

// This method can also be made into a macro, but it should be fine to just duplicate
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

fn create__and_filter(conditions: Vec<Condition>) -> Option<BoxedCondition> {
    create_filter(conditions, AndOr::And)
}

#[test]
fn test() {
    let mut connection = SqliteConnection::establish("file:test?mode=memory&cache=shared").unwrap();

    connection
        .batch_execute(
            r#"
            CREATE TABLE test (
                id TEXT PRIMARY KEY,
                number_field NUMBER NOT NULL,
                text_field TEXT NOT NULL DEFAULT '',
                bool_field BOOL NOT NULL DEFAULT false
            );

            CREATE TABLE join_to_test (
                id TEXT PRIMARY KEY,
                test_id TEXT REFERENCES test(id),
                double_field DOUBLE NOT NULL
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
            .left_join(join_to_test::dsl::join_to_test)
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
            .left_join(join_to_test::dsl::join_to_test)
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
            .left_join(join_to_test::dsl::join_to_test)
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create__and_filter(vec![
        Condition::number_field(NumberFilter::GreaterThen(1)),
        Condition::text_field(StringFilter::Like("%4%".to_string())),
        Condition::bool_field(BooleanFilter::True),
    ])
    .unwrap();
    let result = vec!["4.2".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .left_join(join_to_test::dsl::join_to_test)
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
            Condition::bool_field(BooleanFilter::True),
            Condition::And(vec![
                Condition::bool_field(BooleanFilter::True),
                Condition::bool_field(BooleanFilter::False),
            ]),
        ]),
    ])
    .unwrap();

    let result = vec!["5".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .left_join(join_to_test::dsl::join_to_test)
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create__and_filter(vec![
        Condition::number_field(NumberFilter::Equal(5)),
        Condition::And(vec![
            Condition::Or(vec![
                Condition::bool_field(BooleanFilter::True),
                Condition::bool_field(BooleanFilter::True),
            ]),
            Condition::bool_field(BooleanFilter::False),
        ]),
    ])
    .unwrap();

    // No return
    let result: Vec<String> = vec![];

    assert_eq!(
        result,
        test::dsl::test
            .left_join(join_to_test::dsl::join_to_test)
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    connection
        .batch_execute(
            r#"
            INSERT INTO test 
              (id, number_field) 
            VALUES
              ('6', 6);
            
            INSERT INTO join_to_test 
              (id, test_id, double_field) 
            VALUES
              ('1', 6, 1.2);
        "#,
        )
        .unwrap();

    let condition =
        create__and_filter(vec![Condition::double_field(NumberFilter::Equal(1.2))]).unwrap();

    let result = vec!["6".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .left_join(join_to_test::dsl::join_to_test)
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    connection
        .batch_execute(
            r#"
            INSERT INTO test 
              (id, number_field) 
            VALUES
              ('7.1', 7);

            INSERT INTO test 
              (id, number_field) 
            VALUES
              ('7.2', 7);
            
            INSERT INTO join_to_test 
              (id, test_id, double_field) 
            VALUES
              ('7', '7.1', 0);
        "#,
        )
        .unwrap();

    let condition = create__and_filter(vec![
        Condition::double_field(NumberFilter::IsNull),
        Condition::number_field(NumberFilter::Equal(7)),
    ])
    .unwrap();

    let result = vec!["7.2".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .left_join(join_to_test::dsl::join_to_test)
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create__and_filter(vec![
        Condition::double_field(NumberFilter::IsNotNull),
        Condition::number_field(NumberFilter::Equal(7)),
    ])
    .unwrap();

    let result = vec!["7.1".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .left_join(join_to_test::dsl::join_to_test)
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );
}
