# Konfig

## Variables

You can set variables by writing `<key> = <value>`
Or using the [shortand for nested variables](#scopes) `key::nested = <value>`

Examples:

- `debug = true`
- `ip = 'localhost'`
- `port = 8080`
- `api_key = null`

## Types

A variable can have one of the following types:

- [String](#strings)
- [Integer](#integers)
- [Float](#floats)
- [Null](#null)
- [Array](#arrays)
- [Dict](#dictionaries)

### Strings

Strings are denoted by single colons, and can't have any, unless escaped, new lines

Example: `'content\n'`

### Integers

Integers works like everywhere

Examples:

- `var = 123`
- `var = 0xFF`
- `var = 0o750`

### Float

The only thing that distinguishes integers from floats are their ending  
Floats can't convert non decimal values to other bases

Examples:

- `var = 123.321`

### Null

Null will always represent the word `null`, regardless of the implementation, being it a keyword, or being it an enum

Example: `var = null`

### Arrays

To define an array use `[...]`

Examples:

- `var = [true, false, false]`
- ```
  var = [
      false,
      true,
      true,
  ]
  ```

### Dictionaries

To define an dictionary use `{.key: value}`

Example:

- ```
  task = {
      .name: 'wake up'
      .done: true
      .hours_slept: 7.5
      .next_in: 14.0
  }
  ```

## Scopes

Scopes transforms this:

```
some = {
    .big: {
        .and: {
            .nested: {
                .variable: null
            }
        }
    }
}
```

Into this:
```
some::big::and::nested::variable = null
```

There's no scopes inside dictionaries