# Pokedle
Your second daily dose of Pokemon

## Notes on pyoc (the crate used for the Python bindings)
* Having to add the module by hand and not just tag functions and classes is error prone (easy to forget something, difference between the lib name and module name), it's rust, not C++. For example the wasm bindings works very well without having to define a module like this
* Not beeing to really use enum (can only have c-enum) feels clunky, but I understand that Python can't use union by value (there is maybe another way to do it but haven't found it in the project/doc)
* Documentation is missing some stuff (getter/setter syntax was not totally there, examples miss the __use__ statement, etc.)