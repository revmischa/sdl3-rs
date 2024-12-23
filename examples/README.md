# sdl3-sys: Low level Rust bindings for SDL 3

- This version of [`sdl3-sys`](https://github.com/maia-s/sdl3-sys-rs
) has bindings for SDL version `3.1.6-preview` and earlier.

- SDL 3 is ABI stable as of the 3.1.3 preview release, but `sdl3-sys` is new and may have bugs. Please submit an issue at github if you have any issues or comments!

- [docs.rs/sdl3-sys](https://docs.rs/sdl3-sys/latest/sdl3_sys/)

- Known issues(241223):

> [!CAUTION]
> - Satellite libraries (`mixer`, `image`, `ttf`) aren’t available yet
> - There are no tests yet, except for static asserts translated from the original headers
> - Some less common targets are missing detection or features to enable corresponding SDL features

<hr />

# SDL3 new GPU API merged (Hacker News)
- [SDL3 is still in preview, but the new GPU API is now merged into the main branch while SDL3 maintainers apply some final tweaks(240930)](https://news.ycombinator.com/item?id=41396260)
    - Unreal/Unity are not the only solutions. There is also bgfx (C/C++)(https://github.com/bkaradzic/bgfx),
    - which is quite popular and sokol gfx (C)(https://github.com/floooh/sokol) 
    - which I know of. Of course there are many more lesser known ones.

# libsdl-org news(the latest news)
- https://news.ycombinator.com/from?site=github.com/libsdl-org

- https://hn.algolia.com/?q=sdl3 

- https://hn.algolia.com/?dateRange=pastYear&page=0&prefix=false&query=sdl3&sort=byPopularity&type=story 