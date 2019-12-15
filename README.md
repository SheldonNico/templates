# Some useful templates.
- rust-to-cmake: rust call static cpp library. (1. use cmake crate auto compile 2. ar collect all dependencies into one static library 3. if rust want use cpp, the best way is through ``extern C`` , or use cpp crate which provide more traits to wrap cpp into rust(and it did work, but you should include/link dir properly, it's extremely difficult if you have 3rd library), (but IMO, i don't like mixing too code in same library).)

