/*
    Complex joins
*/

use diesel::{
    dsl::Eq,
    helper_types::{InnerJoin, InnerJoinQuerySource, IntoBoxed, LeftJoinOn, LeftJoinQuerySource},
    prelude::*,
    sql_types::{Bool, Nullable},
    sqlite::Sqlite,
};

use super::*;
use crate::*;

table! {
    road (id) {
        id -> Text,
        name -> Text
    }
}

table! {
    cycle_lane (id) {
        id -> Text,
        name -> Text,
        road_id -> Text
    }
}

table! {
    bike_trip (id) {
        id -> Text,
        name -> Text,
        bike_id -> Text,
        road_id -> Text
    }
}

use bike::bike as bike_table;
joinable!(bike_trip -> bike_table (bike_id));
allow_tables_to_appear_in_same_query!(bike_trip, cycle_lane, bike_table);

#[allow(non_camel_case_types)]
pub(super) enum Condition {
    bike(StringFilter),
    cycle_lane(StringFilter),
    bike_trip(StringFilter),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

type CycleLaneRoadIdEqBikeTripRoadId = Eq<cycle_lane::dsl::road_id, bike_trip::dsl::road_id>;

type ConditionSource = LeftJoinQuerySource<
    InnerJoinQuerySource<bike_trip::dsl::bike_trip, bike_table::dsl::bike>,
    cycle_lane::dsl::cycle_lane,
    CycleLaneRoadIdEqBikeTripRoadId,
>;
type BoxedCondition = Box<dyn BoxableExpression<ConditionSource, Sqlite, SqlType = Nullable<Bool>>>;

type QuerySource = LeftJoinOn<
    InnerJoin<bike_trip::dsl::bike_trip, bike_table::dsl::bike>,
    cycle_lane::dsl::cycle_lane,
    CycleLaneRoadIdEqBikeTripRoadId,
>;
type BoxedQuery = IntoBoxed<'static, QuerySource, Sqlite>;

pub(super) fn create_filtered_query(conditions: Vec<Condition>) -> BoxedQuery {
    let boxed_query = bike_trip::dsl::bike_trip
        .inner_join(bike_table::dsl::bike)
        // Skipping road and just joining on road_id
        .left_join(
            cycle_lane::dsl::cycle_lane.on(cycle_lane::dsl::road_id.eq(bike_trip::dsl::road_id)),
        )
        .into_boxed();

    match create_filter(conditions, AndOr::And) {
        Some(boxed_conditions) => boxed_query.filter(boxed_conditions),
        None => boxed_query,
    }
}

impl Condition {
    fn to_boxed_condition(self) -> Option<BoxedCondition> {
        Some(match self {
            Condition::bike(f) => string_filter!(f, bike_table::dsl::name),
            Condition::cycle_lane(f) => string_filter!(f, cycle_lane::dsl::name),
            Condition::bike_trip(f) => string_filter!(f, bike_trip::dsl::name),
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
