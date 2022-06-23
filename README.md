# Intrusive List in Rust

A simple implementation of an intrusive linked list in Rust.
I made it the safest I could, knowing that for it to work correctly
reading any of the items besides the head's list (which is valid always
it's in the `Some` variant) needs exclusive access to the list in order
to be thread safe.
