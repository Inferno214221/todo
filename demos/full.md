# /home/inferno214221/Programming/todo/src/main.rs

## // TODO: 

- [ ] \[Ln 12, Col 0]
```rs
use walkdir::{DirEntry, IntoIter, WalkDir};
use colored::{Color, ColoredString, Colorize, Styles};

```
`// TODO: `**check all lifetime specifiers**
```rs
// TODO: check all unwrap / expect usages
// TODO: check that \r\n is supported
// ? should I use lines() rather than split('\n')?
```

## // TODO: 

- [ ] \[Ln 13, Col 0]
```rs
use colored::{Color, ColoredString, Colorize, Styles};

// TODO: check all lifetime specifiers
```
`// TODO: `**check all unwrap / expect usages**
```rs
// TODO: check that \r\n is supported
// ? should I use lines() rather than split('\n')?
// ? when should I use explicit typing?
```

## // TODO: 

- [ ] \[Ln 14, Col 0]
```rs

// TODO: check all lifetime specifiers
// TODO: check all unwrap / expect usages
```
`// TODO: `**check that \r\n is supported**
```rs
// ? should I use lines() rather than split('\n')?
// ? when should I use explicit typing?

```

## // TODO: 

- [ ] \[Ln 276, Col 0]
```rs

fn generate_output_for_file(file_found_patterns: &FileFoundPatterns, args: &Args) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
```
`    // TODO: `**can't handle matches on the first line kinda need to add 0 and the end to this array**
```rs
    let mut is_first_pattern: bool = true;

    for found_pattern in &file_found_patterns.found_patterns {
```

## // ? 

- [ ] \[Ln 15, Col 0]
```rs
// TODO: check all lifetime specifiers
// TODO: check all unwrap / expect usages
// TODO: check that \r\n is supported
```
`// ? `**should I use lines() rather than split('\n')?**
```rs
// ? when should I use explicit typing?

#[derive(Debug)]
```

## // ? 

- [ ] \[Ln 16, Col 0]
```rs
// TODO: check all unwrap / expect usages
// TODO: check that \r\n is supported
// ? should I use lines() rather than split('\n')?
```
`// ? `**when should I use explicit typing?**
```rs

#[derive(Debug)]
struct Args {
```

## // ? 

- [ ] \[Ln 259, Col 0]
```rs
                contents,
                found_patterns,
            });
```
`        } // ? `**use match and throw an error on read fail**
```rs
        return None;
    }).collect::<Vec<FileFoundPatterns<'a>>>();
}
```

