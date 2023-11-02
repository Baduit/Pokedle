# Pokedle
Your second daily dose of Pokemon

You can play here: https://pokedle.baduit.eu/

I was inspired by [Loldle](https://loldle.net/classic) to do it, but when I had the idea [Pokédle](https://pokedle.net/classic) was not created yet. After I worked on this (some time passed), I checked and now it exists. I decided to finish it anyway, and at least it made me practice a bit rust and learn to use [pyo3](https://crates.io/crates/pyo3).

## Notes on pyoc (the crate used for the Python bindings)
* Having to add the module by hand and not just tag functions and classes is error prone (easy to forget something, difference between the lib name and module name), it's rust, not C++. For example the wasm bindings works very well without having to define a module like this
* Not beeing to really use enum (can only have c-enum) feels clunky, but I understand that Python can't use union by value (there is maybe another way to do it but haven't found it in the project/doc)
* Documentation is missing some stuff (getter/setter syntax was not totally there, examples miss the __use__ statement, etc.)
* Using strong typing in the interface is a pain in the ass, instead of beeing able to create bindings for my rust code, it seems that the way to use it is to design my rust code to adapt the library, that's not right, I must be doing something wrong here, I can't believe otherwise this is the most used way to create python bindings for rust.