use diesel::{connection::SimpleConnection, prelude::*, SqliteConnection};

use crate::inner_statement::bike::create_filtered_query;

mod bike;
mod bike_trip;
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
    use super::*;

    {
        use self::person::*;

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
    connection
        .batch_execute(
            r#"
            CREATE TABLE road (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );

            CREATE TABLE bike_trip (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                road_id TEXT REFERENCES road(id),
                bike_id TEXT REFERENCES bike(id)
            );

            CREATE TABLE cycle_lane (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                road_id TEXT REFERENCES road(id)
            );

            INSERT INTO road 
              (id, name) 
            VALUES
              ('tamaki', 'tamaki');
            
            INSERT INTO road 
              (id, name) 
            VALUES
              ('queen', 'queen');

            INSERT INTO cycle_lane 
              (id, name, road_id) 
            VALUES
              ('upper_queen_lane', 'upper_queen_lane', 'queen');

            INSERT INTO cycle_lane 
              (id, name, road_id) 
            VALUES
              ('lower_queen_lane', 'lower_queen_lane', 'queen');
            
            INSERT INTO cycle_lane 
              (id, name, road_id) 
            VALUES
              ('bendy', 'bendy', 'tamaki');

            INSERT INTO cycle_lane 
              (id, name, road_id) 
            VALUES
              ('flatty', 'flatty', 'tamaki');

            INSERT INTO cycle_lane 
              (id, name, road_id) 
            VALUES
              ('windy', 'windy', 'tamaki');    

            
            INSERT INTO bike_trip 
              (id, name, road_id, bike_id) 
            VALUES
              ('t1', 't1', 'queen', 'c1');   

            INSERT INTO bike_trip 
              (id, name, road_id, bike_id) 
            VALUES
              ('t2', 't2', 'tamaki', 'm1');   
        "#,
        )
        .unwrap();

    use self::bike_trip::*;

    let condition = vec![Condition::bike(StringFilter::Equal("m1".to_string()))];
    let result = vec![
        Some("bendy".to_string()),
        Some("flatty".to_string()),
        Some("windy".to_string()),
    ];

    assert_eq!(
        result,
        create_filtered_query(condition)
            .select(cycle_lane::dsl::name.nullable())
            .order_by(cycle_lane::dsl::name)
            .load::<Option<String>>(&mut connection)
            .unwrap()
    );
}
