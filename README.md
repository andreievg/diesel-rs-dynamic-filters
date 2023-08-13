# Examples of dynamic filtering with [diesel-rs](https://github.com/diesel-rs/diesel)

### Dynamic Filtering ?

Quite often we want to build filters at runtime (i.e. from Graphql input), this requires a structure of defining dynamic filter shape and a mechanism to convert it to database queries.

### Motivation

Anything dynamic in diesel-rs can be quite daunting at the beginning, thus I wanted to show examples of dynamic filtering with diesel-rs via evolutionary guide (from basic to complex).

### Content

This tutorial is meant to be consumed in steps, each step has a branch and each step evolves from the previous step (git diff helps with understanding the progression). 
These topics are covered:

* Basic condition boxing
* Generic type filters
* And/Or and nesting/grouping condition
* Joins
* Macros to help with extendability
* Inner queries
* Complex joins

### Generics and Types

I tried to find a balance between the use of generic and complexity, and in some cases opted in for macros or copy paste approach. 
When working in a team, readability and simplicity trumps most other subjective characteristics of code, and I think complex use of generics undermines that characteristic.

## Basic condition boxing

We start with [basic-generic-filters branch](https://github.com/andreievg/diesel-rs-dynamic-filters/tree/basic-generic-filters) and [this file](https://github.com/andreievg/diesel-rs-dynamic-filters/blob/basic-generic-filters/src/dynamic_filters.rs).

Basically we get to below data structure:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/74283f9093f4f85147b1cf13c1f05707e0ad29c6/src/dynamic_filters.rs#L15-L20

That can be used in the following way:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/74283f9093f4f85147b1cf13c1f05707e0ad29c6/src/dynamic_filters.rs#L116-L127

## Generic type filters

The next step, in [generic-field-filters branch](https://github.com/andreievg/diesel-rs-dynamic-filters/tree/generic-field-filters) and as per [this diff](https://github.com/andreievg/diesel-rs-dynamic-filters/compare/basic-generic-filters...generic-field-filters), generic type filters were added, to be able to filter more then one field.

This allowed this structure:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/d715883c337f11336390c8cc16423b2f292420db/src/dynamic_filters.rs#L17-L23

To be used in the following way:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/d715883c337f11336390c8cc16423b2f292420db/src/dynamic_filters.rs#L158-L162

## And/Or and nesting/grouping condition

This is where it starts to get interesting, in [and/or branch](https://github.com/andreievg/diesel-rs-dynamic-filters/tree/and/or) as per [this diff](https://github.com/andreievg/diesel-rs-dynamic-filters/compare/generic-field-filters...and/or) we are able to use dynamic and and or statement with unlimited number of nesting and fine grained control over the grouping.

The tests have a good example of this, basically we get:

### bool_field = true or (bool_field = true and bool_field = false) === true:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/b8591aa2534bd00e5c9956581b67db35d3cfe019/src/dynamic_filters.rs#L217-L224

### (bool_field = true or bool_field = true) and bool_field = false === false:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/b8591aa2534bd00e5c9956581b67db35d3cfe019/src/dynamic_filters.rs#L240-L247

## Joins

The next step was to show how we can build dynamic conditions for joined tables, in [joins branch](https://github.com/andreievg/diesel-rs-dynamic-filters/tree/joins) as per [this diff.](https://github.com/andreievg/diesel-rs-dynamic-filters/compare/and/or...joins)
You will note in the following statement, BoxedCondition now returns `Nullable` bool, and `.nullable()` needs to be added to every single field, this doesn't affect the condition, and macros will help us with the syntax.

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/babe4f0d6f11ea71d04dbaecf02ffe97bd8ee0d4/src/dynamic_filters.rs#L64

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/babe4f0d6f11ea71d04dbaecf02ffe97bd8ee0d4/src/dynamic_filters.rs#L74

An example of more complex joins will be shown in later section

## Macros

It should be easy to extend existing condition and add filtering functionality to new tables, thus helper were added in [macros branch](https://github.com/andreievg/diesel-rs-dynamic-filters/tree/macros) and [this diff](https://github.com/andreievg/diesel-rs-dynamic-filters/compare/joins...macros).

Now new filter can be added by one line addition of a field in conditions enum and a case in match:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/9c938aeb4ffc6a8725e921d0b223856aa7483857/src/dynamic_filters.rs#L40

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/9c938aeb4ffc6a8725e921d0b223856aa7483857/src/dynamic_filters.rs#L55

In addition, is_null was added to boolean filter type to show how we can enforce that filter:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/9c938aeb4ffc6a8725e921d0b223856aa7483857/src/dynamic_filters.rs#L327

Even with macros there seems to be a bit of bloat in code and to add dynamic filtering functionality to a new table we would need to add `Condition` enum, `Impl` of that `Condition` enum and `create_filter` method, it would still be just copy paster operation, I thought that was a good compromise for readability, since a macro to auto generate this method or to make it a method with generics would add a bit of complexity.

## Inner query

Sometimes you may want to re-use existing queries and filters as a condition elsewhere. 
The [inner-query branch](https://github.com/andreievg/diesel-rs-dynamic-filters/tree/inner-query) as per [this diff](https://github.com/andreievg/diesel-rs-dynamic-filters/compare/macros...inner-query), makes this possible by also creating a boxed query.

You would have seen the use of diesel type [helpers already](https://docs.rs/diesel/latest/diesel/helper_types/index.html), here the use is extended to create boxed query with filter, that is later used as a condition in another table.

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/776eca21be5f8c91d210fff622d6139f497d9be9/src/inner_statement/bike.rs#L77-L84

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/776eca21be5f8c91d210fff622d6139f497d9be9/src/inner_statement/person.rs#L45-L53

That results in being able to:

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/776eca21be5f8c91d210fff622d6139f497d9be9/src/inner_statement/mod.rs#L78-L80

This example also shows that macros make extension of field type filters quite easy

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/776eca21be5f8c91d210fff622d6139f497d9be9/src/lib.rs#L30

https://github.com/andreievg/diesel-rs-dynamic-filters/blob/776eca21be5f8c91d210fff622d6139f497d9be9/src/lib.rs#L39

TODO is there a way to use one 'source' (ConditionSource and QuerySource)

## Complex joins

One of the harder things I found with diesel-rs types was construction boxed join types, especially when join has ON statements as showcase in [complex-joins branch](https://github.com/andreievg/diesel-rs-dynamic-filters/tree/complex-joins) and [this diff](https://github.com/andreievg/diesel-rs-dynamic-filters/compare/inner-query...complex-joins).

You can extend this example by adding `road_on_bike_ride` join table, rather than `road_id` on `bike_ride` table (this will allow multiple roads to be attached to a bike_ride and a good exercise for diesel-rs query boxing)

## Summary

I hope you found this tutorial useful, you can create an issue if you need some clarification or found an error, etc..

(For anyone that's interested, the original trigger for writing this tutorial came from a more demanding filtering requirement from [omSupply project](https://github.com/openmsupply/open-msupply) during a central server synchronization research done in one of our monthly RnD days)
