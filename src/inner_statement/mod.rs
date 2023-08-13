use diesel::{connection::SimpleConnection, prelude::*, SqliteConnection};

use crate::inner_statement::bike::create_filtered_query;

mod bike;
mod person;

#[test]
fn test() {
    let mut connection = SqliteConnection::establish("file:test?mode=memory&cache=shared").unwrap();

    connection
        .batch_execute(
            r#"
            CREATE TABLE person (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );

            CREATE TABLE color (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );

            CREATE TABLE bike (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                owner_id TEXT REFERENCES person(id),
                color_id TEXT REFERENCES color(id)
            );

            INSERT INTO color 
              (id, name) 
            VALUES
              ('orange', 'orange');
            
            INSERT INTO color 
              (id, name) 
            VALUES
              ('purple', 'purple');
            
            INSERT INTO color 
              (id, name) 
            VALUES
              ('grey', 'grey');

            INSERT INTO person 
              (id, name) 
            VALUES
              ('craig', 'craig');
            
            INSERT INTO person 
              (id, name) 
            VALUES
              ('mark', 'mark');

            INSERT INTO bike 
              (id, name, owner_id, color_id) 
            VALUES
              ('c1', 'c1', 'craig', 'orange');
            
            INSERT INTO bike 
              (id, name, owner_id, color_id) 
            VALUES
              ('c2', 'c2', 'craig', 'purple');

            INSERT INTO bike 
              (id, name, owner_id, color_id) 
            VALUES
              ('m1', 'm1', 'mark', 'grey');
        "#,
        )
        .unwrap();

    use self::person::*;
    use super::*;

    let condition = vec![Condition::bike(vec![bike::Condition::color(
        StringFilter::In(vec!["orange".to_string(), "purple".to_string()]),
    )])];
    let result = vec!["craig".to_string()];

    assert_eq!(
        result,
        create_filtered_query(condition)
            .select(person::dsl::id)
            .load::<String>(&mut connection)
            .unwrap()
    );
}
