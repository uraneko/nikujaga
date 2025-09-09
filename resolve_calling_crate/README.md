<h1>resolve_calling_crate</h1>
this is a small crate that resolves the path of the currently running crate.
Useful for envoking file reading functionalities inside workspace crates

This crate exists because sometimes a crate A that has a function to read some file in the cwd would be used as a dependency of a crate B. 
Assuming that A naively expects the cwd to be set to B and that B is a member of some workspace W, 
then the cwd would be set to the workspace cwd and A would fail at its read; either by reading a different file with the same name as what was asked of it, or, by not finding the specified file to read. 
