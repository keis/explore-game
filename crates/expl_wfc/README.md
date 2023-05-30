# Random grid generator using Wave Function Collapse

A strategy for generating random grids from a input sample so that each `tile`
(consisting of a central cell and the adjacent cells) of the output is also
present in the input.

## Example

With a input like this with edges wrapping around:

```
     ^ % % ~ ~
    ^ ^ % % ~ %
   % % % % % % %
  % % % % ^ ^ % %
 % % ^ ^ ^ ^ % % %
  % ^ % ^ ~ % % %
   % % % ~ ~ % %
    % % ~ ~ % %
     % % ~ ~ %
```

A possible output is (seed: AEKAV7N5IMULPIHHFACA):

```
 % % % % % % % % % ^ % % % % ~ % % % % %
  % % % % % % ^ ^ ^ % % % % % % % ~ % % ~
 ~ ^ ^ % % % % % ^ % % ~ ~ % % % ~ ~ % %
  ^ ^ % % ~ % % % % ~ ~ ~ % % % % % % ^ ^
 ^ % % % ~ ~ % % % % ~ ~ % % ~ % % % % ^
  ^ % % % % % ^ ^ % % ~ ^ % ~ ~ % ^ % % ~
 ~ ~ ~ % % % % ^ % % ^ ^ ^ % % % ^ % % ~
  ~ ~ % % ^ % % ^ ^ ^ ^ % ^ % % ^ ^ ^ ~ ~
 ~ ~ ~ % % ^ % % % ^ ~ % % % ~ ~ ~ ^ ^ ^
  ~ % % % ^ % % % % ~ ~ % % % ~ ~ % % ^ %
```

The example uses a simple `char`s as the cell value but more complex type can be
used as long as they are `Copy`, `Hash`, `Eq`, and `Ord`

## Implementation

The WFC algorithm works in two phases

1. A [template](src/template.rs) is prepared from the input that describes what
   tiles can go next to each other.
2. The output is constructed through a process of elimination by the [generator](src/generator.rs)
