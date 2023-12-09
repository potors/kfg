before pull run and fix:
  - cargo clippy
  - cargo fmt
  - [cargo spellcheck](https://github.com/drahnr/cargo-spellcheck)
  - [cargo bloat](https://github.com/RazrFalcon/cargo-bloat)

should do:
  - [ ] ~~impl serde~~
  - [ ] escaped characters inside strings
  - [ ] refactor parser
  - [x] tests
    - [x] parser
    - [x] tokenizer
    - [x] filter
  - [x] testing is too hard, ignore for a while
  - [x] fix \n as 0th character
