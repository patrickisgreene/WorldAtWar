# World At War

World At War is a real-time strategy game set on a 1:1 scale replica of Earth, utilizing NASA geographic data for terrain and elevation. Players command global military forces, managing ballistic missiles, aircraft, and supply lines in a high-stakes conflict. The game emphasizes realistic military strategy and resource management

### Build earth data
```bash
cargo run --bin preprocess_earth -p waw_earth_preprocess --release
```

### Examples
##### Bash
```bash
cargo run --example earth
```

### Inspiration
 - https://github.com/SebLague/Geographical-Adventures
 - https://github.com/kurtkuehnert/planetary_terrain_renderer
 - https://blog.graysonhead.net/posts/bevy-proc-earth-1/