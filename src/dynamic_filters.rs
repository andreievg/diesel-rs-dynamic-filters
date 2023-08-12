/*
    Simple generic condition for one table and one number_field.
*/

use diesel::{connection::SimpleConnection, prelude::*, sql_types::Bool, sqlite::Sqlite};

table! {
    test (id) {
        id -> Text,
        number_field -> Integer
    }
}

// Filters for "number_field"
enum Condition {
    Equal(i32),
    NotEqual(i32),
    GreaterThen(i32),
    LowerThen(i32),
}
// Need this type for common condition expressions
type BoxedCondition = Box<dyn BoxableExpression<test::dsl::test, Sqlite, SqlType = Bool>>;

fn create_filter(conditions: Vec<Condition>) -> Option<BoxedCondition> {
    conditions
        .into_iter()
        // Map into array of boxed conditions
        .map::<BoxedCondition, _>(|condition| match condition {
            Condition::Equal(value) => Box::new(test::dsl::number_field.eq(value)),
            Condition::NotEqual(value) => Box::new(test::dsl::number_field.ne(value)),
            Condition::GreaterThen(value) => Box::new(test::dsl::number_field.gt(value)),
            Condition::LowerThen(value) => Box::new(test::dsl::number_field.lt(value)),
        })
        // Reduce to a boxed_condition1.and(boxed_condition2).and(boxed_condition3)...
        .fold(
            None,
            |boxed_conditions, boxed_condition| match boxed_conditions {
                Some(bc) => Some(Box::new(bc.and(boxed_condition))),
                None => Some(boxed_condition),
            },
        )

    // OR
    // let mut boxed_conditions: Option<BoxedCondition> = None;
    // for condition in conditions {
    //     let boxed_condition = match condition {
    //         Condition::Equal(value) => Box::new(test::dsl::number_field.eq(value)),
    //         Condition::NotEqual(value) => Box::new(test::dsl::number_field.eq(value)),
    //         Condition::GreaterThen(value) => Box::new(test::dsl::number_field.eq(value)),
    //         Condition::LowerThen(value) => Box::new(test::dsl::number_field.eq(value)),
    //     };

    //     boxed_conditions = match boxed_conditions {
    //         Some(bc) => Some(Box::new(bc.and(boxed_condition))),
    //         None => Some(boxed_condition),
    //     };
    // }
    // boxed_conditions
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
                number_field NUMBER NOT NULL
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

    let condition = create_filter(vec![Condition::Equal(1)]).unwrap();
    let result = vec!["1".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );

    let condition = create_filter(vec![Condition::NotEqual(1)]).unwrap();
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

    let condition =
        create_filter(vec![Condition::GreaterThen(1), Condition::LowerThen(3)]).unwrap();
    let result = vec!["2".to_string()];

    assert_eq!(
        result,
        test::dsl::test
            .filter(condition)
            .select(test::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );
}
