# libflowd-rs

Parse and marshal [libflowd v2 frames](https://github.com/ERnsTL/flowd), enabling the creation of ```flowd``` components in Rust.


## Optimization Potentials

* Nom parser can likely be further optimized (matchers, inlining, allocations ...)
* actually return IPs from Nom parser
* return how many bytes were consumed (nice for debug outputs)
* Reduce allocations in conventional parser -- how far is that possible?
	  It must hand out copies of the results unfortunately, but by packing the parser in a struct, it could re-use allocated state and counter variables.


## Insights Regarding the Nom Parser

Nom - like most parsers - is not a streaming parser. It does not keep state, does not automatically refill its buffer.

* [example how to use the support around it to make it streaming](https://github.com/Geal/generator_nom)
* [another example](https://stackoverflow.com/questions/46876879/how-do-i-create-a-streaming-parser-in-nom)

TODO there is a Producer + Consumer construct in Nom ... check that out.


## Insights Regarding Rust Ownership w.r.t. Parsers

Generally, in Rust, it is not possible to hand out references to something out of a function unless that is directly derived from the input variables.

Handing out result references which point into the buffer of the BufReader would be zero-copy, but is impossible because the buffer will be overwritten at some point and because Rust does not recognize that the buffer is transitively derived from the owned BufReader.

* [reasoning and explanation](https://stackoverflow.com/questions/35664419/how-do-i-duplicate-a-u8-slice)

May be improved in the future when the Rust compiler can known that inside the BufReader there is a buffer which this function owns.


## Benchmarks

Run them with:

```
rustc -C opt-level=3 src/lib.rs --test -o bench && ./bench --bench
```

or simply

```
cargo bench
```


## Tests

Run them with:

```
cargo test --lib -- --nocapture --test-threads=1
```

or simply

```
cargo test
```


## References

Parsing using Nom (as of 2018-12):

* [Nom introduction](https://stevedonovan.github.io/rust-gentle-intro/nom-intro.html)
* [another Nom introduction](http://siciarz.net/24-days-rust-nom-part-1/)
* [original streaming Nom parser + generator example using Nom v3.x](https://github.com/Geal/generator_nom/blob/master/src/main.rs) - probably also possible using Iterator trait instead of generator. Advantage generator: Distinction between yield and complete state.
* [upgrading to Nom 4.0](https://github.com/Geal/nom/blob/master/doc/upgrading_to_nom_4.md)
* [Nom parsers and matchers as examples called "flavors"](https://github.com/rust-av/flavors/blob/master/src/parser.rs)
* [list of Nom parsers](https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md)
* [InputTakeAtPosition trait](https://github.com/Geal/nom/blob/a88a6f8d3fe3cf41ef682bbfec1fd72f08edcf95/src/traits.rs#L499) which is used in many parsers
* [alphanumeric parser using InputTakeAtPosition as example](https://github.com/Geal/snack/blob/415a689bcdd6b0a3a4d61365237bee9c45dd2f91/src/nom.rs#L186)
